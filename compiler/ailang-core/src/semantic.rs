use crate::ast::{
    BinaryOp, Block, Expr, ExprKind, ItemKind, MatchArm, Pattern, PatternKind, Program, Stmt,
    StmtKind, TypeExpr, TypeExprKind, UnaryOp,
};
use crate::diagnostics::{Diagnostic, Span};
use crate::policy::Policy;
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Type {
    Named { name: String, args: Vec<Type> },
    Unit,
    Unknown,
}

impl Type {
    fn named(name: impl Into<String>) -> Self {
        Self::Named {
            name: name.into(),
            args: Vec::new(),
        }
    }

    fn option(inner: Type) -> Self {
        Self::Named {
            name: "Option".to_string(),
            args: vec![inner],
        }
    }

    fn result(ok: Type, err: Type) -> Self {
        Self::Named {
            name: "Result".to_string(),
            args: vec![ok, err],
        }
    }

    fn describe(&self) -> String {
        match self {
            Type::Named { name, args } if args.is_empty() => name.clone(),
            Type::Named { name, args } => {
                let args_text = args
                    .iter()
                    .map(Type::describe)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{name}<{args_text}>")
            }
            Type::Unit => "Unit".to_string(),
            Type::Unknown => "<unknown>".to_string(),
        }
    }

    fn is_bool(&self) -> bool {
        matches!(self, Type::Named { name, args } if name == "Bool" && args.is_empty())
    }

    fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::Named { name, args }
            if args.is_empty()
                && matches!(name.as_str(), "Int" | "Int64" | "Float64" | "Decimal")
        )
    }

    fn is_named(&self, expected: &str) -> bool {
        matches!(self, Type::Named { name, args } if name == expected && args.is_empty())
    }

    fn contains_named(&self, target: &str) -> bool {
        match self {
            Type::Named { name, args } => {
                name == target || args.iter().any(|arg| arg.contains_named(target))
            }
            Type::Unit | Type::Unknown => false,
        }
    }

    fn contains_secret(&self) -> bool {
        self.contains_named("Secret")
    }

    fn contains_untrusted(&self) -> bool {
        self.contains_named("Untrusted")
    }

    fn compatible_with(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            (Type::Unit, Type::Unit) => true,
            (
                Type::Named {
                    name: a_name,
                    args: a_args,
                },
                Type::Named {
                    name: b_name,
                    args: b_args,
                },
            ) => {
                a_name == b_name
                    && a_args.len() == b_args.len()
                    && a_args
                        .iter()
                        .zip(b_args)
                        .all(|(left, right)| left.compatible_with(right))
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
struct FunctionSig {
    params: Vec<Type>,
    return_type: Type,
    declared_effects: HashSet<String>,
}

#[derive(Debug, Clone)]
struct EnumVariantInfo {
    name: String,
    payload: Vec<TypeExpr>,
}

#[derive(Debug, Clone)]
struct EnumInfo {
    variants: Vec<EnumVariantInfo>,
}

#[derive(Debug, Clone)]
struct StructInfo;

#[derive(Debug, Default)]
struct Catalog {
    primitive_types: HashSet<String>,
    generic_types: HashMap<String, usize>,
    enums: HashMap<String, EnumInfo>,
    structs: HashMap<String, StructInfo>,
    functions: HashMap<String, FunctionSig>,
    known_effects: HashSet<String>,
}

impl Catalog {
    fn new() -> Self {
        let primitive_types = [
            "Bool",
            "Int",
            "Int64",
            "Float64",
            "Decimal",
            "String",
            "Bytes",
            "Time",
            "Duration",
            "Uuid",
            "Unit",
            "DbCap",
            "TxCap",
            "NetCap",
            "InternalNetCap",
            "FsCap",
            "SecretsCap",
            "SqlQuery",
            "HtmlSafe",
            "PublicUrl",
            "InternalUrl",
            "PathSafe",
            "HeaderName",
            "HeaderValue",
            "Cookie",
            "LogValue",
            "Budget",
            "StdError",
            "Origin",
        ]
        .into_iter()
        .map(|item| item.to_string())
        .collect::<HashSet<_>>();

        let generic_types = [
            ("Option".to_string(), 1),
            ("Result".to_string(), 2),
            ("List".to_string(), 1),
            ("Map".to_string(), 2),
            ("Set".to_string(), 1),
            ("Secret".to_string(), 1),
            ("Untrusted".to_string(), 1),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        Self {
            primitive_types,
            generic_types,
            enums: HashMap::new(),
            structs: HashMap::new(),
            functions: HashMap::new(),
            known_effects: [
                "log",
                "time.now",
                "net",
                "secrets.read",
                "secrets.reveal",
                "db.read",
                "db.write",
                "fs.read",
                "fs.write",
                "shell",
                "unsafe",
            ]
            .into_iter()
            .map(|item| item.to_string())
            .collect::<HashSet<_>>(),
        }
    }

    fn is_known_type_name(&self, name: &str) -> bool {
        self.primitive_types.contains(name)
            || self.generic_types.contains_key(name)
            || self.enums.contains_key(name)
            || self.structs.contains_key(name)
    }
}

struct Analyzer {
    catalog: Catalog,
    policy: Policy,
    diagnostics: Vec<Diagnostic>,
}

pub fn analyze_program(program: &Program) -> Result<(), Vec<Diagnostic>> {
    analyze_program_with_policy(program, &Policy::default())
}

pub fn analyze_program_with_policy(
    program: &Program,
    policy: &Policy,
) -> Result<(), Vec<Diagnostic>> {
    let mut analyzer = Analyzer {
        catalog: Catalog::new(),
        policy: policy.clone(),
        diagnostics: Vec::new(),
    };

    analyzer.collect_types(program);
    analyzer.collect_functions(program);
    analyzer.check_type_references(program);
    analyzer.check_function_bodies(program);

    if analyzer.diagnostics.is_empty() {
        Ok(())
    } else {
        analyzer
            .diagnostics
            .sort_by(|left, right| left.code.cmp(&right.code));
        Err(analyzer.diagnostics)
    }
}

impl Analyzer {
    fn collect_types(&mut self, program: &Program) {
        for item in &program.items {
            match &item.kind {
                ItemKind::Struct(decl) => {
                    if self.catalog.structs.contains_key(&decl.name)
                        || self.catalog.enums.contains_key(&decl.name)
                        || self.catalog.primitive_types.contains(&decl.name)
                        || self.catalog.generic_types.contains_key(&decl.name)
                    {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "N3002",
                                "duplicate type declaration",
                                item.span.clone(),
                            )
                            .with_note(format!("type `{}` is already declared", decl.name)),
                        );
                    } else {
                        self.catalog.structs.insert(decl.name.clone(), StructInfo);
                    }
                }
                ItemKind::Enum(decl) => {
                    if self.catalog.structs.contains_key(&decl.name)
                        || self.catalog.enums.contains_key(&decl.name)
                        || self.catalog.primitive_types.contains(&decl.name)
                        || self.catalog.generic_types.contains_key(&decl.name)
                    {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "N3002",
                                "duplicate type declaration",
                                item.span.clone(),
                            )
                            .with_note(format!("type `{}` is already declared", decl.name)),
                        );
                    } else {
                        let variants = decl
                            .variants
                            .iter()
                            .map(|variant| EnumVariantInfo {
                                name: variant.name.clone(),
                                payload: variant
                                    .payload
                                    .iter()
                                    .map(|field| field.ty.clone())
                                    .collect(),
                            })
                            .collect::<Vec<_>>();
                        self.catalog
                            .enums
                            .insert(decl.name.clone(), EnumInfo { variants });
                    }
                }
                ItemKind::Function(_) => {}
            }
        }
    }

    fn collect_functions(&mut self, program: &Program) {
        for item in &program.items {
            let ItemKind::Function(function) = &item.kind else {
                continue;
            };

            if self.catalog.functions.contains_key(&function.name) {
                self.diagnostics.push(
                    Diagnostic::error("N3002", "duplicate function declaration", item.span.clone())
                        .with_note(format!("function `{}` is already declared", function.name)),
                );
                continue;
            }

            let params = function
                .params
                .iter()
                .map(|param| self.resolve_type_expr(&param.ty, param.span.clone()))
                .collect::<Vec<_>>();

            let return_type = function
                .return_type
                .as_ref()
                .map(|ty| self.resolve_type_expr(ty, ty.span.clone()))
                .unwrap_or(Type::Unit);

            let mut declared_effects = HashSet::new();
            for effect in &function.effects {
                let effect_name = effect.as_name();
                if !self.catalog.known_effects.contains(&effect_name) {
                    self.diagnostics.push(
                        Diagnostic::error("E4001", "unknown effect name", effect.span.clone())
                            .with_note(format!("effect `{effect_name}` is not recognized")),
                    );
                    continue;
                }

                if self.policy.forbidden_effects.contains(&effect_name) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E2002",
                            "effect forbidden by policy",
                            effect.span.clone(),
                        )
                        .with_note(format!(
                            "effect `{effect_name}` is forbidden by the active policy"
                        )),
                    );
                }

                if !declared_effects.insert(effect_name.clone()) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E4003",
                            "duplicate effect declaration",
                            effect.span.clone(),
                        )
                        .with_note(format!("effect `{effect_name}` is declared more than once")),
                    );
                }
            }

            self.catalog.functions.insert(
                function.name.clone(),
                FunctionSig {
                    params,
                    return_type,
                    declared_effects,
                },
            );
        }
    }

    fn check_type_references(&mut self, program: &Program) {
        for item in &program.items {
            match &item.kind {
                ItemKind::Struct(decl) => {
                    for field in &decl.fields {
                        self.resolve_type_expr(&field.ty, field.span.clone());
                    }
                }
                ItemKind::Enum(decl) => {
                    for variant in &decl.variants {
                        for payload in &variant.payload {
                            self.resolve_type_expr(&payload.ty, payload.span.clone());
                        }
                    }
                }
                ItemKind::Function(function) => {
                    for param in &function.params {
                        self.resolve_type_expr(&param.ty, param.span.clone());
                    }
                    if let Some(return_type) = &function.return_type {
                        self.resolve_type_expr(return_type, return_type.span.clone());
                    }
                }
            }
        }
    }

    fn check_function_bodies(&mut self, program: &Program) {
        for item in &program.items {
            let ItemKind::Function(function) = &item.kind else {
                continue;
            };

            let signature = self
                .catalog
                .functions
                .get(&function.name)
                .cloned()
                .unwrap_or(FunctionSig {
                    params: Vec::new(),
                    return_type: Type::Unknown,
                    declared_effects: HashSet::new(),
                });

            let mut env = HashMap::new();
            for (index, param) in function.params.iter().enumerate() {
                if env.contains_key(&param.name) {
                    self.diagnostics.push(
                        Diagnostic::error("N3002", "duplicate parameter name", param.span.clone())
                            .with_note(format!("parameter `{}` is already defined", param.name)),
                    );
                    continue;
                }

                let param_type = signature
                    .params
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| self.resolve_type_expr(&param.ty, param.span.clone()));
                env.insert(param.name.clone(), param_type);
            }

            let mut used_effects = HashSet::new();
            let body_type = self.analyze_block(
                &function.body,
                &mut env,
                &signature.return_type,
                &mut used_effects,
            );

            if !signature.return_type.compatible_with(&body_type) {
                self.diagnostics.push(
                    Diagnostic::error(
                        "T3105",
                        "function body type does not match declared return type",
                        function.body.span.clone(),
                    )
                    .with_note(format!(
                        "declared `{}`, but block evaluates to `{}`",
                        signature.return_type.describe(),
                        body_type.describe()
                    )),
                );
            }

            let undeclared = used_effects
                .difference(&signature.declared_effects)
                .cloned()
                .collect::<BTreeSet<_>>();
            for effect in undeclared {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4002",
                        "effect used but not declared",
                        function.body.span.clone(),
                    )
                    .with_note(format!(
                        "function `{}` uses `{effect}` but does not declare it in `effects {{ ... }}`",
                        function.name
                    )),
                );
            }

            let forbidden_used = used_effects
                .intersection(&self.policy.forbidden_effects)
                .cloned()
                .collect::<BTreeSet<_>>();
            for effect in forbidden_used {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E2002",
                        "effect forbidden by policy",
                        function.body.span.clone(),
                    )
                    .with_note(format!(
                        "function `{}` uses forbidden effect `{effect}` under active policy",
                        function.name
                    )),
                );
            }
        }
    }

    fn analyze_block(
        &mut self,
        block: &Block,
        env: &mut HashMap<String, Type>,
        expected_return: &Type,
        used_effects: &mut HashSet<String>,
    ) -> Type {
        let mut scoped = env.clone();

        for stmt in &block.statements {
            self.analyze_statement(stmt, &mut scoped, expected_return, used_effects);
        }

        if let Some(tail) = &block.tail {
            self.analyze_expr(tail, &mut scoped, used_effects)
        } else {
            Type::Unit
        }
    }

    fn analyze_statement(
        &mut self,
        stmt: &Stmt,
        env: &mut HashMap<String, Type>,
        expected_return: &Type,
        used_effects: &mut HashSet<String>,
    ) {
        match &stmt.kind {
            StmtKind::Let {
                name, ty, value, ..
            } => {
                let value_type = self.analyze_expr(value, env, used_effects);
                let bound_type = if let Some(annotation) = ty {
                    let annotation_type =
                        self.resolve_type_expr(annotation, annotation.span.clone());
                    if !annotation_type.compatible_with(&value_type) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "T3101",
                                "type mismatch in variable binding",
                                stmt.span.clone(),
                            )
                            .with_note(format!(
                                "variable `{name}` expects `{}`, got `{}`",
                                annotation_type.describe(),
                                value_type.describe()
                            )),
                        );
                    }
                    annotation_type
                } else {
                    value_type
                };

                env.insert(name.clone(), bound_type);
            }
            StmtKind::Return { value } => {
                let return_type = if let Some(value) = value {
                    self.analyze_expr(value, env, used_effects)
                } else {
                    Type::Unit
                };

                if !expected_return.compatible_with(&return_type) {
                    self.diagnostics.push(
                        Diagnostic::error("T3105", "return type mismatch", stmt.span.clone())
                            .with_note(format!(
                                "expected `{}`, got `{}`",
                                expected_return.describe(),
                                return_type.describe()
                            )),
                    );
                }
            }
            StmtKind::Expr { expr } => {
                self.analyze_expr(expr, env, used_effects);
            }
        }
    }

    fn analyze_expr(
        &mut self,
        expr: &Expr,
        env: &mut HashMap<String, Type>,
        used_effects: &mut HashSet<String>,
    ) -> Type {
        match &expr.kind {
            ExprKind::Identifier(name) => env.get(name).cloned().unwrap_or_else(|| {
                self.diagnostics.push(
                    Diagnostic::error("N3003", "unknown identifier", expr.span.clone())
                        .with_note(format!("`{name}` is not defined in this scope")),
                );
                Type::Unknown
            }),
            ExprKind::Number(text) => {
                if text.contains('.') {
                    Type::named("Float64")
                } else {
                    Type::named("Int")
                }
            }
            ExprKind::String(_) => Type::named("String"),
            ExprKind::Bool(_) => Type::named("Bool"),
            ExprKind::Unary { op, expr: inner } => {
                let inner_type = self.analyze_expr(inner, env, used_effects);
                match op {
                    UnaryOp::Neg => {
                        if !inner_type.is_numeric() {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "T3101",
                                    "unary `-` expects a numeric expression",
                                    expr.span.clone(),
                                )
                                .with_note(format!("found `{}`", inner_type.describe())),
                            );
                            Type::Unknown
                        } else {
                            inner_type
                        }
                    }
                    UnaryOp::Not => {
                        if !inner_type.is_bool() {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "T3101",
                                    "unary `!` expects a boolean expression",
                                    expr.span.clone(),
                                )
                                .with_note(format!("found `{}`", inner_type.describe())),
                            );
                            Type::Unknown
                        } else {
                            Type::named("Bool")
                        }
                    }
                }
            }
            ExprKind::Binary { op, left, right } => {
                let left_type = self.analyze_expr(left, env, used_effects);
                let right_type = self.analyze_expr(right, env, used_effects);

                match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Rem => {
                        if !left_type.is_numeric() || !right_type.is_numeric() {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "T3101",
                                    "arithmetic operation expects numeric operands",
                                    expr.span.clone(),
                                )
                                .with_note(format!(
                                    "left=`{}`, right=`{}`",
                                    left_type.describe(),
                                    right_type.describe()
                                )),
                            );
                            Type::Unknown
                        } else if !left_type.compatible_with(&right_type) {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "T3101",
                                    "arithmetic operands must have compatible types",
                                    expr.span.clone(),
                                )
                                .with_note(format!(
                                    "left=`{}`, right=`{}`",
                                    left_type.describe(),
                                    right_type.describe()
                                )),
                            );
                            left_type
                        } else {
                            left_type
                        }
                    }
                    BinaryOp::Eq
                    | BinaryOp::Ne
                    | BinaryOp::Lt
                    | BinaryOp::Le
                    | BinaryOp::Gt
                    | BinaryOp::Ge => {
                        if !left_type.compatible_with(&right_type) {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "T3101",
                                    "comparison operands must have compatible types",
                                    expr.span.clone(),
                                )
                                .with_note(format!(
                                    "left=`{}`, right=`{}`",
                                    left_type.describe(),
                                    right_type.describe()
                                )),
                            );
                        }
                        Type::named("Bool")
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if !left_type.is_bool() || !right_type.is_bool() {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "T3101",
                                    "logical operation expects boolean operands",
                                    expr.span.clone(),
                                )
                                .with_note(format!(
                                    "left=`{}`, right=`{}`",
                                    left_type.describe(),
                                    right_type.describe()
                                )),
                            );
                        }
                        Type::named("Bool")
                    }
                }
            }
            ExprKind::Member { object, .. } => {
                let _ = self.analyze_expr(object, env, used_effects);
                Type::Unknown
            }
            ExprKind::Call { callee, args } => {
                self.analyze_call(expr.span.clone(), callee, args, env, used_effects)
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_type = self.analyze_expr(condition, env, used_effects);
                if !condition_type.is_bool() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "T3102",
                            "if condition must be boolean",
                            condition.span.clone(),
                        )
                        .with_note(format!("found `{}`", condition_type.describe())),
                    );
                }

                let then_type = self.analyze_block(then_branch, env, &Type::Unknown, used_effects);
                if let Some(else_expr) = else_branch {
                    let else_type = self.analyze_expr(else_expr, env, used_effects);
                    if !then_type.compatible_with(&else_type) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "T3101",
                                "if branches must produce compatible types",
                                expr.span.clone(),
                            )
                            .with_note(format!(
                                "then=`{}`, else=`{}`",
                                then_type.describe(),
                                else_type.describe()
                            )),
                        );
                    }
                    then_type
                } else {
                    Type::Unit
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                let scrutinee_type = self.analyze_expr(scrutinee, env, used_effects);
                self.analyze_match(expr.span.clone(), &scrutinee_type, arms, env, used_effects)
            }
            ExprKind::Block(block) => self.analyze_block(block, env, &Type::Unknown, used_effects),
        }
    }

    fn analyze_call(
        &mut self,
        span: Span,
        callee: &Expr,
        args: &[Expr],
        env: &mut HashMap<String, Type>,
        used_effects: &mut HashSet<String>,
    ) -> Type {
        let Some(name) = callable_name(callee) else {
            self.diagnostics.push(
                Diagnostic::error(
                    "T3104",
                    "unsupported callable expression",
                    callee.span.clone(),
                )
                .with_note("only named functions/constructors are callable in v0.1-lite"),
            );
            for arg in args {
                self.analyze_expr(arg, env, used_effects);
            }
            return Type::Unknown;
        };

        let arg_types = args
            .iter()
            .map(|arg| self.analyze_expr(arg, env, used_effects))
            .collect::<Vec<_>>();
        self.enforce_sink_flow_restrictions(name.as_str(), args, &arg_types);

        if let Some(signature) = self.catalog.functions.get(name.as_str()).cloned() {
            for effect in &signature.declared_effects {
                used_effects.insert(effect.clone());
            }

            if signature.params.len() != args.len() {
                self.diagnostics.push(
                    Diagnostic::error("T3103", "function argument count mismatch", span).with_note(
                        format!(
                            "`{name}` expects {}, got {}",
                            signature.params.len(),
                            args.len()
                        ),
                    ),
                );
            }

            for (index, arg) in args.iter().enumerate() {
                let arg_type = &arg_types[index];
                if let Some(expected) = signature.params.get(index) {
                    if !expected.compatible_with(arg_type) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "T3101",
                                "function argument type mismatch",
                                arg.span.clone(),
                            )
                            .with_note(format!(
                                "argument {} expects `{}`, got `{}`",
                                index + 1,
                                expected.describe(),
                                arg_type.describe()
                            )),
                        );
                    }
                }
            }

            return signature.return_type;
        }

        if let Some(intrinsic) = intrinsic_spec_for(name.as_str()) {
            used_effects.insert(intrinsic.effect.to_string());

            if let Some(required_capability) = intrinsic.required_capability {
                match arg_types.first() {
                    None => {
                        self.diagnostics.push(
                            Diagnostic::error("E2003", "operation requires capability", span.clone())
                                .with_note(format!(
                                "`{name}` requires first argument capability `{required_capability}`"
                                )),
                        );
                    }
                    Some(actual) if !actual.is_named(required_capability) => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "E2004",
                                "capability type mismatch",
                                args[0].span.clone(),
                            )
                            .with_note(format!(
                                "`{name}` expects capability `{required_capability}`, got `{}`",
                                actual.describe()
                            )),
                        );
                    }
                    Some(_) => {}
                }
            }

            return Type::Unit;
        }

        if let Some(constructor) =
            self.resolve_builtin_constructor(name.as_str(), span.clone(), args, env, used_effects)
        {
            return constructor;
        }

        if self.catalog.structs.contains_key(name.as_str()) {
            if !args.is_empty() {
                self.diagnostics.push(
                    Diagnostic::error(
                        "T3103",
                        "struct constructor does not accept arguments",
                        span,
                    )
                    .with_note(format!("`{name}()` expects 0 arguments")),
                );
            }

            return Type::named(name);
        }

        if let Some((owner_enum, payload_types)) = self.find_enum_variant_constructor(name.as_str())
        {
            if payload_types.len() != args.len() {
                self.diagnostics.push(
                    Diagnostic::error("T3103", "enum constructor argument count mismatch", span)
                        .with_note(format!(
                            "variant `{name}` expects {}, got {}",
                            payload_types.len(),
                            args.len()
                        )),
                );
            }

            for (index, arg) in args.iter().enumerate() {
                let arg_type = &arg_types[index];
                if let Some(expected) = payload_types.get(index) {
                    if !expected.compatible_with(arg_type) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "T3101",
                                "enum constructor argument type mismatch",
                                arg.span.clone(),
                            )
                            .with_note(format!(
                                "variant `{name}` argument {} expects `{}`, got `{}`",
                                index + 1,
                                expected.describe(),
                                arg_type.describe()
                            )),
                        );
                    }
                }
            }

            return Type::named(owner_enum);
        }

        self.diagnostics.push(
            Diagnostic::error(
                "T3104",
                "unknown function or constructor",
                callee.span.clone(),
            )
            .with_note(format!("`{name}` is not declared")),
        );
        Type::Unknown
    }

    fn analyze_match(
        &mut self,
        span: Span,
        scrutinee_type: &Type,
        arms: &[MatchArm],
        env: &mut HashMap<String, Type>,
        used_effects: &mut HashSet<String>,
    ) -> Type {
        let mut seen_bool_true = false;
        let mut seen_bool_false = false;
        let mut seen_variants = HashSet::new();
        let mut has_catch_all = false;

        let mut arm_result: Option<Type> = None;

        for arm in arms {
            let mut arm_env = env.clone();
            let coverage = self.bind_pattern(&arm.pattern, scrutinee_type, &mut arm_env);
            match coverage {
                PatternCoverage::BoolTrue => seen_bool_true = true,
                PatternCoverage::BoolFalse => seen_bool_false = true,
                PatternCoverage::Variant(name) => {
                    seen_variants.insert(name);
                }
                PatternCoverage::CatchAll => has_catch_all = true,
                PatternCoverage::Other => {}
            }

            let value_type = self.analyze_expr(&arm.value, &mut arm_env, used_effects);
            if let Some(existing) = &arm_result {
                if !existing.compatible_with(&value_type) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "T3101",
                            "match arms must produce compatible types",
                            arm.span.clone(),
                        )
                        .with_note(format!(
                            "previous arm type=`{}`, this arm type=`{}`",
                            existing.describe(),
                            value_type.describe()
                        )),
                    );
                }
            } else {
                arm_result = Some(value_type);
            }
        }

        if !has_catch_all {
            if scrutinee_type.is_bool() && (!seen_bool_true || !seen_bool_false) {
                self.diagnostics.push(
                    Diagnostic::error("T3107", "non-exhaustive match", span.clone())
                        .with_note("boolean matches must cover `true` and `false`"),
                );
            }

            if let Some((required, _payload)) = self.enum_variants_for_type(scrutinee_type) {
                let missing = required
                    .into_iter()
                    .filter(|variant| !seen_variants.contains(variant))
                    .collect::<Vec<_>>();

                if !missing.is_empty() {
                    self.diagnostics.push(
                        Diagnostic::error("T3107", "non-exhaustive match", span)
                            .with_note(format!("missing variants: {}", missing.join(", "))),
                    );
                }
            }
        }

        arm_result.unwrap_or(Type::Unit)
    }

    fn enforce_sink_flow_restrictions(
        &mut self,
        callee_name: &str,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        let start_index = sink_user_arg_start_index(callee_name);
        for (index, (arg, arg_type)) in args.iter().zip(arg_types).enumerate() {
            if index < start_index {
                continue;
            }

            if is_log_sink(callee_name) {
                if arg_type.contains_secret() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1003",
                            "secret value cannot be logged",
                            arg.span.clone(),
                        )
                        .with_note(format!("sink `{callee_name}` rejects `Secret<_>` values"))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(arg, arg_type))
                        .with_note("use `redact(secret)` or remove the secret from log payload"),
                    );
                } else if arg_type.contains_untrusted() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1002",
                            "untrusted value cannot flow into log sink",
                            arg.span.clone(),
                        )
                        .with_note(format!("sink `{callee_name}` requires trusted log values"))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(arg, arg_type))
                        .with_note("validate/sanitize input before constructing log payload"),
                    );
                }
                continue;
            }

            if is_json_sink(callee_name) {
                if arg_type.contains_secret() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1004",
                            "secret value cannot be JSON-encoded",
                            arg.span.clone(),
                        )
                        .with_note(format!("sink `{callee_name}` rejects `Secret<_>` values"))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(arg, arg_type))
                        .with_note("return a redacted or derived non-secret value"),
                    );
                } else if arg_type.contains_untrusted() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1002",
                            "untrusted value cannot flow into JSON response sink",
                            arg.span.clone(),
                        )
                        .with_note(format!("sink `{callee_name}` requires trusted values"))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(arg, arg_type))
                        .with_note("decode/validate input with schema before encoding"),
                    );
                }
                continue;
            }

            if is_sql_sink(callee_name) {
                if arg_type.contains_secret() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1005",
                            "secret value cannot flow into SQL sink",
                            arg.span.clone(),
                        )
                        .with_note(format!(
                            "sink `{callee_name}` rejects `Secret<_>` values"
                        ))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(arg, arg_type))
                        .with_note("use non-secret identifiers/values when building SqlQuery"),
                    );
                } else if arg_type.contains_untrusted() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1002",
                            "untrusted value cannot flow into SQL sink",
                            arg.span.clone(),
                        )
                        .with_note(format!(
                            "sink `{callee_name}` requires trusted SQL inputs"
                        ))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(arg, arg_type))
                        .with_note("validate input and construct typed `SqlQuery`"),
                    );
                }
                continue;
            }

            if is_url_sink(callee_name) {
                if arg_type.contains_secret() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1005",
                            "secret value cannot flow into URL sink",
                            arg.span.clone(),
                        )
                        .with_note(format!(
                            "sink `{callee_name}` rejects `Secret<_>` values"
                        ))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(arg, arg_type))
                        .with_note("derive a non-secret `PublicUrl`/`InternalUrl` through URL validation gates"),
                    );
                } else if arg_type.contains_untrusted() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1002",
                            "untrusted value cannot flow into URL sink",
                            arg.span.clone(),
                        )
                        .with_note(format!(
                            "sink `{callee_name}` requires typed safe URLs"
                        ))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(arg, arg_type))
                        .with_note("validate input via `url.public(...)` / `url.internal(...)`"),
                    );
                }
                continue;
            }

            if is_fs_sink(callee_name) {
                if arg_type.contains_secret() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1005",
                            "secret value cannot flow into filesystem sink",
                            arg.span.clone(),
                        )
                        .with_note(format!(
                            "sink `{callee_name}` rejects `Secret<_>` values"
                        ))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(arg, arg_type))
                        .with_note("use non-secret `PathSafe` values for filesystem operations"),
                    );
                } else if arg_type.contains_untrusted() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1002",
                            "untrusted value cannot flow into filesystem sink",
                            arg.span.clone(),
                        )
                        .with_note(format!(
                            "sink `{callee_name}` requires trusted `PathSafe` values"
                        ))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(arg, arg_type))
                        .with_note("validate input via `path.under(...)` before filesystem access"),
                    );
                }
                continue;
            }

            if is_header_sink(callee_name) {
                if arg_type.contains_secret() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1005",
                            "secret value cannot flow into header/cookie sink",
                            arg.span.clone(),
                        )
                        .with_note(format!(
                            "sink `{callee_name}` rejects `Secret<_>` values"
                        ))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(arg, arg_type))
                        .with_note("use redacted/derived values and validated header or cookie builders"),
                    );
                } else if arg_type.contains_untrusted() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1002",
                            "untrusted value cannot flow into header/cookie sink",
                            arg.span.clone(),
                        )
                        .with_note(format!(
                            "sink `{callee_name}` requires validated header/cookie values"
                        ))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(arg, arg_type))
                        .with_note("validate via `validate.headerValue(...)` or typed cookie builders"),
                    );
                }
            }
        }
    }

    fn bind_pattern(
        &mut self,
        pattern: &Pattern,
        expected: &Type,
        env: &mut HashMap<String, Type>,
    ) -> PatternCoverage {
        match &pattern.kind {
            PatternKind::Wildcard => PatternCoverage::CatchAll,
            PatternKind::Identifier(name) => {
                if name == "_" {
                    return PatternCoverage::CatchAll;
                }

                if let Some((variants, _)) = self.enum_variants_for_type(expected) {
                    if variants.iter().any(|variant| variant == name) {
                        return PatternCoverage::Variant(name.clone());
                    }
                }

                env.insert(name.clone(), expected.clone());
                PatternCoverage::CatchAll
            }
            PatternKind::Variant { name, args } => {
                let Some((variants, payload_map)) = self.enum_variants_for_type(expected) else {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "T3108",
                            "variant pattern requires enum/option/result type",
                            pattern.span.clone(),
                        )
                        .with_note(format!("found `{}`", expected.describe())),
                    );
                    return PatternCoverage::Other;
                };

                if !variants.iter().any(|candidate| candidate == name) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "T3109",
                            "unknown variant in pattern",
                            pattern.span.clone(),
                        )
                        .with_note(format!(
                            "variant `{name}` is not part of `{}`",
                            expected.describe()
                        )),
                    );
                    return PatternCoverage::Other;
                }

                let expected_payload = payload_map.get(name).cloned().unwrap_or_default();

                if expected_payload.len() != args.len() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "T3103",
                            "pattern argument count mismatch",
                            pattern.span.clone(),
                        )
                        .with_note(format!(
                            "variant `{name}` expects {}, got {}",
                            expected_payload.len(),
                            args.len()
                        )),
                    );
                }

                for (index, arg_pattern) in args.iter().enumerate() {
                    let expected_arg_type = expected_payload
                        .get(index)
                        .cloned()
                        .unwrap_or(Type::Unknown);
                    self.bind_pattern(arg_pattern, &expected_arg_type, env);
                }

                PatternCoverage::Variant(name.clone())
            }
            PatternKind::Number(_) => {
                if !Type::named("Int").compatible_with(expected) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "T3108",
                            "number pattern type mismatch",
                            pattern.span.clone(),
                        )
                        .with_note(format!("expected `{}`", expected.describe())),
                    );
                }
                PatternCoverage::Other
            }
            PatternKind::String(_) => {
                if !Type::named("String").compatible_with(expected) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "T3108",
                            "string pattern type mismatch",
                            pattern.span.clone(),
                        )
                        .with_note(format!("expected `{}`", expected.describe())),
                    );
                }
                PatternCoverage::Other
            }
            PatternKind::Bool(value) => {
                if !Type::named("Bool").compatible_with(expected) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "T3108",
                            "boolean pattern type mismatch",
                            pattern.span.clone(),
                        )
                        .with_note(format!("expected `{}`", expected.describe())),
                    );
                }

                if *value {
                    PatternCoverage::BoolTrue
                } else {
                    PatternCoverage::BoolFalse
                }
            }
        }
    }

    fn resolve_type_expr(&mut self, expr: &TypeExpr, span: Span) -> Type {
        match &expr.kind {
            TypeExprKind::Named { name, args } => {
                if !self.catalog.is_known_type_name(name) {
                    self.diagnostics.push(
                        Diagnostic::error("N3001", "unknown type", span)
                            .with_note(format!("type `{name}` is not declared")),
                    );
                    return Type::Unknown;
                }

                if let Some(expected_arity) = self.catalog.generic_types.get(name) {
                    if args.len() != *expected_arity {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "N3004",
                                "generic type argument count mismatch",
                                expr.span.clone(),
                            )
                            .with_note(format!(
                                "`{name}` expects {expected_arity} type argument(s), got {}",
                                args.len()
                            )),
                        );
                    }
                } else if !args.is_empty() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "N3004",
                            "non-generic type used with type arguments",
                            expr.span.clone(),
                        )
                        .with_note(format!("`{name}` does not accept type arguments")),
                    );
                }

                let resolved_args = args
                    .iter()
                    .map(|arg| self.resolve_type_expr(arg, arg.span.clone()))
                    .collect::<Vec<_>>();

                if name == "Unit" && !resolved_args.is_empty() {
                    Type::Unknown
                } else {
                    Type::Named {
                        name: name.clone(),
                        args: resolved_args,
                    }
                }
            }
        }
    }

    fn resolve_builtin_constructor(
        &mut self,
        name: &str,
        span: Span,
        args: &[Expr],
        env: &mut HashMap<String, Type>,
        used_effects: &mut HashSet<String>,
    ) -> Option<Type> {
        match name {
            "Some" => {
                if args.len() != 1 {
                    self.diagnostics.push(
                        Diagnostic::error("T3103", "constructor argument count mismatch", span)
                            .with_note("`Some` expects exactly one argument"),
                    );
                    return Some(Type::option(Type::Unknown));
                }
                let inner = self.analyze_expr(&args[0], env, used_effects);
                Some(Type::option(inner))
            }
            "None" => {
                if !args.is_empty() {
                    self.diagnostics.push(
                        Diagnostic::error("T3103", "constructor argument count mismatch", span)
                            .with_note("`None` expects 0 arguments"),
                    );
                }
                for arg in args {
                    self.analyze_expr(arg, env, used_effects);
                }
                Some(Type::option(Type::Unknown))
            }
            "Ok" => {
                if args.len() != 1 {
                    self.diagnostics.push(
                        Diagnostic::error("T3103", "constructor argument count mismatch", span)
                            .with_note("`Ok` expects exactly one argument"),
                    );
                    for arg in args {
                        self.analyze_expr(arg, env, used_effects);
                    }
                    return Some(Type::result(Type::Unknown, Type::Unknown));
                }
                let ok = self.analyze_expr(&args[0], env, used_effects);
                Some(Type::result(ok, Type::Unknown))
            }
            "Err" => {
                if args.len() != 1 {
                    self.diagnostics.push(
                        Diagnostic::error("T3103", "constructor argument count mismatch", span)
                            .with_note("`Err` expects exactly one argument"),
                    );
                    for arg in args {
                        self.analyze_expr(arg, env, used_effects);
                    }
                    return Some(Type::result(Type::Unknown, Type::Unknown));
                }
                let err = self.analyze_expr(&args[0], env, used_effects);
                Some(Type::result(Type::Unknown, err))
            }
            _ => None,
        }
    }

    fn find_enum_variant_constructor(&mut self, variant_name: &str) -> Option<(String, Vec<Type>)> {
        for (enum_name, enum_info) in self.catalog.enums.clone() {
            for variant in &enum_info.variants {
                if variant.name == variant_name {
                    let payload = variant
                        .payload
                        .iter()
                        .map(|expr| self.resolve_type_expr(expr, expr.span.clone()))
                        .collect::<Vec<_>>();
                    return Some((enum_name, payload));
                }
            }
        }

        None
    }

    fn enum_variants_for_type(
        &mut self,
        ty: &Type,
    ) -> Option<(Vec<String>, HashMap<String, Vec<Type>>)> {
        match ty {
            Type::Named { name, args } if name == "Option" => {
                let inner = args.first().cloned().unwrap_or(Type::Unknown);
                let mut payload = HashMap::new();
                payload.insert("Some".to_string(), vec![inner]);
                payload.insert("None".to_string(), Vec::new());
                Some((vec!["Some".to_string(), "None".to_string()], payload))
            }
            Type::Named { name, args } if name == "Result" => {
                let ok = args.first().cloned().unwrap_or(Type::Unknown);
                let err = args.get(1).cloned().unwrap_or(Type::Unknown);
                let mut payload = HashMap::new();
                payload.insert("Ok".to_string(), vec![ok]);
                payload.insert("Err".to_string(), vec![err]);
                Some((vec!["Ok".to_string(), "Err".to_string()], payload))
            }
            Type::Named { name, args } if args.is_empty() => {
                let enum_info = self.catalog.enums.get(name).cloned()?;
                let mut variants = Vec::new();
                let mut payload = HashMap::new();

                for variant in enum_info.variants {
                    variants.push(variant.name.clone());
                    let payload_types = variant
                        .payload
                        .iter()
                        .map(|expr| self.resolve_type_expr(expr, expr.span.clone()))
                        .collect::<Vec<_>>();
                    payload.insert(variant.name.clone(), payload_types);
                }

                Some((variants, payload))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct IntrinsicSpec {
    effect: &'static str,
    required_capability: Option<&'static str>,
}

fn intrinsic_spec_for(name: &str) -> Option<IntrinsicSpec> {
    match name {
        "log" | "log.emit" | "log.info" | "log.warn" | "log.error" => Some(IntrinsicSpec {
            effect: "log",
            required_capability: None,
        }),
        "req_json" | "req.json" => Some(IntrinsicSpec {
            effect: "net",
            required_capability: None,
        }),
        "res_json" | "res.json" => Some(IntrinsicSpec {
            effect: "net",
            required_capability: None,
        }),
        "res_html" | "res.html" => Some(IntrinsicSpec {
            effect: "net",
            required_capability: None,
        }),
        "set_header" | "res.setHeader" => Some(IntrinsicSpec {
            effect: "net",
            required_capability: None,
        }),
        "set_cookie" | "res.addCookie" => Some(IntrinsicSpec {
            effect: "net",
            required_capability: None,
        }),
        "time_now" | "time.now" => Some(IntrinsicSpec {
            effect: "time.now",
            required_capability: None,
        }),
        "net_call" | "httpClient.get" => Some(IntrinsicSpec {
            effect: "net",
            required_capability: Some("NetCap"),
        }),
        "net_internal_call" | "httpClient.getInternal" => Some(IntrinsicSpec {
            effect: "net",
            required_capability: Some("InternalNetCap"),
        }),
        "secret_read" | "secrets.get" => Some(IntrinsicSpec {
            effect: "secrets.read",
            required_capability: Some("SecretsCap"),
        }),
        "secret_reveal" | "secrets.reveal" => Some(IntrinsicSpec {
            effect: "secrets.reveal",
            required_capability: Some("SecretsCap"),
        }),
        "db_read" | "db.queryOne" => Some(IntrinsicSpec {
            effect: "db.read",
            required_capability: Some("DbCap"),
        }),
        "db_write" | "db.exec" => Some(IntrinsicSpec {
            effect: "db.write",
            required_capability: Some("DbCap"),
        }),
        "fs_read" | "fs.read" => Some(IntrinsicSpec {
            effect: "fs.read",
            required_capability: Some("FsCap"),
        }),
        "fs_write" | "fs.write" => Some(IntrinsicSpec {
            effect: "fs.write",
            required_capability: Some("FsCap"),
        }),
        _ => None,
    }
}

fn callable_name(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name.clone()),
        ExprKind::Member { object, field } => {
            let mut prefix = callable_name(object)?;
            prefix.push('.');
            prefix.push_str(field);
            Some(prefix)
        }
        _ => None,
    }
}

fn is_log_sink(name: &str) -> bool {
    matches!(
        name,
        "log" | "log.emit" | "log.info" | "log.warn" | "log.error"
    )
}

fn is_json_sink(name: &str) -> bool {
    matches!(name, "res_json" | "res.json")
}

fn is_sql_sink(name: &str) -> bool {
    matches!(name, "db_write" | "db.exec" | "db_read" | "db.queryOne")
}

fn is_url_sink(name: &str) -> bool {
    matches!(
        name,
        "net_call" | "httpClient.get" | "net_internal_call" | "httpClient.getInternal"
    )
}

fn is_fs_sink(name: &str) -> bool {
    matches!(name, "fs_read" | "fs.read" | "fs_write" | "fs.write")
}

fn is_header_sink(name: &str) -> bool {
    matches!(
        name,
        "set_header" | "res.setHeader" | "set_cookie" | "res.addCookie"
    )
}

fn sink_user_arg_start_index(name: &str) -> usize {
    if matches!(
        name,
        "db_write"
            | "db.exec"
            | "db_read"
            | "db.queryOne"
            | "fs_read"
            | "fs.read"
            | "fs_write"
            | "fs.write"
            | "net_call"
            | "httpClient.get"
            | "net_internal_call"
            | "httpClient.getInternal"
            | "secret_read"
            | "secrets.get"
            | "secret_reveal"
            | "secrets.reveal"
    ) {
        1
    } else {
        0
    }
}

fn flow_origin_note(expr: &Expr, ty: &Type) -> String {
    match &expr.kind {
        ExprKind::Identifier(name) => {
            format!(
                "origin: identifier `{name}` carries type `{}`",
                ty.describe()
            )
        }
        ExprKind::Call { callee, .. } => {
            if let Some(name) = callable_name(callee) {
                format!("origin: value comes from call `{name}(...)`")
            } else {
                format!(
                    "origin: value comes from call expression of type `{}`",
                    ty.describe()
                )
            }
        }
        ExprKind::Member { .. } => {
            format!(
                "origin: value comes from member expression of type `{}`",
                ty.describe()
            )
        }
        _ => format!("origin: expression has type `{}`", ty.describe()),
    }
}

#[derive(Debug, Clone)]
enum PatternCoverage {
    BoolTrue,
    BoolFalse,
    Variant(String),
    CatchAll,
    Other,
}
