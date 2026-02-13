use crate::ast::{
    BinaryOp, Block, Expr, ExprKind, FunctionDecl, ItemKind, MatchArm, Pattern, PatternKind,
    Program, Stmt, StmtKind, TypeExpr, TypeExprKind, UnaryOp,
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

    fn untrusted(inner: Type) -> Self {
        Self::Named {
            name: "Untrusted".to_string(),
            args: vec![inner],
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

    fn is_untrusted_string(&self) -> bool {
        matches!(
            self,
            Type::Named { name, args }
                if name == "Untrusted"
                    && args.len() == 1
                    && args[0].is_named("String")
        )
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
            "Json",
            "Bytes",
            "Time",
            "Duration",
            "Uuid",
            "Email",
            "Unit",
            "Ctx",
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
            "LogAttr",
            "LogEvent",
            "LogValue",
            "Budget",
            "StdError",
            "Origin",
            "OriginPattern",
            "CorsOrigins",
            "CorsConfig",
            "SecurityHeadersConfig",
            "CsrfConfig",
            "AuthConfig",
            "CspConfig",
            "CspPolicy",
            "Principal",
            "Caps",
            "Router",
            "Request",
            "Response",
            "HttpError",
            "Handler",
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
            ("Schema".to_string(), 1),
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
                "db.tx",
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
    callable_forward_summaries: HashMap<String, String>,
    value_origins: HashMap<String, String>,
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
        callable_forward_summaries: HashMap::new(),
        value_origins: HashMap::new(),
        diagnostics: Vec::new(),
    };

    analyzer.collect_types(program);
    analyzer.collect_functions(program);
    analyzer.build_callable_forward_summaries(program);
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
                        .with_tag("security")
                        .with_tag("policy")
                        .with_tag("effects")
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

    fn build_callable_forward_summaries(&mut self, program: &Program) {
        let functions = program
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                ItemKind::Function(function) => Some(function),
                _ => None,
            })
            .collect::<Vec<_>>();

        let mut summaries = HashMap::new();
        let max_rounds = functions.len().max(1);
        for _ in 0..max_rounds {
            let mut changed = false;
            for function in &functions {
                let next = self.infer_function_callable_forward(function, &summaries);
                match next {
                    Some(target) => {
                        if summaries.get(function.name.as_str()) != Some(&target) {
                            summaries.insert(function.name.clone(), target);
                            changed = true;
                        }
                    }
                    None => {
                        if summaries.remove(function.name.as_str()).is_some() {
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        self.callable_forward_summaries = summaries;
    }

    fn infer_function_callable_forward(
        &self,
        function: &FunctionDecl,
        summaries: &HashMap<String, String>,
    ) -> Option<String> {
        let mut callable_aliases = seed_callable_aliases_from_param_type_exprs(&function.params);
        for stmt in &function.body.statements {
            if let StmtKind::Let { name, value, .. } = &stmt.kind {
                if let Some(target) =
                    self.infer_callable_forward_expr(value, &callable_aliases, summaries)
                {
                    callable_aliases.insert(name.clone(), target);
                } else {
                    callable_aliases.remove(name);
                }
            }
        }

        function
            .body
            .tail
            .as_ref()
            .and_then(|expr| self.infer_callable_forward_expr(expr, &callable_aliases, summaries))
    }

    fn infer_callable_forward_expr(
        &self,
        expr: &Expr,
        callable_aliases: &HashMap<String, String>,
        summaries: &HashMap<String, String>,
    ) -> Option<String> {
        if let Some(name) = resolve_callable_name(expr, callable_aliases) {
            return self.normalize_callable_forward_target(name, summaries, true);
        }

        if let ExprKind::Call { callee, .. } = &expr.kind {
            let callee_name = resolve_callable_name(callee, callable_aliases)?;
            return self.normalize_callable_forward_target(callee_name, summaries, false);
        }

        None
    }

    fn normalize_callable_forward_target(
        &self,
        name: String,
        summaries: &HashMap<String, String>,
        allow_function_fallback: bool,
    ) -> Option<String> {
        let resolved = resolve_summary_name(name, summaries);
        if intrinsic_spec_for(resolved.as_str()).is_some()
            || is_intrinsic_namespace(resolved.as_str())
        {
            Some(resolved)
        } else if allow_function_fallback && self.catalog.functions.contains_key(resolved.as_str())
        {
            Some(resolved)
        } else {
            None
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
            let mut callable_aliases =
                seed_callable_aliases_from_params(&function.params, &signature.params);
            self.value_origins.clear();
            let body_type = self.analyze_block(
                &function.body,
                &mut env,
                &signature.return_type,
                &mut used_effects,
                &mut callable_aliases,
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
                    .with_tag("effects")
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
                    .with_tag("security")
                    .with_tag("policy")
                    .with_tag("effects")
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
        callable_aliases: &mut HashMap<String, String>,
    ) -> Type {
        let mut scoped = env.clone();
        let mut scoped_aliases = callable_aliases.clone();
        let outer_value_origins = self.value_origins.clone();

        for stmt in &block.statements {
            self.analyze_statement(
                stmt,
                &mut scoped,
                expected_return,
                used_effects,
                &mut scoped_aliases,
            );
        }

        let result = if let Some(tail) = &block.tail {
            self.analyze_expr(tail, &mut scoped, used_effects, &mut scoped_aliases)
        } else {
            Type::Unit
        };
        self.value_origins = outer_value_origins;
        result
    }

    fn analyze_statement(
        &mut self,
        stmt: &Stmt,
        env: &mut HashMap<String, Type>,
        expected_return: &Type,
        used_effects: &mut HashSet<String>,
        callable_aliases: &mut HashMap<String, String>,
    ) {
        match &stmt.kind {
            StmtKind::Let {
                name, ty, value, ..
            } => {
                let value_type = self.analyze_expr(value, env, used_effects, callable_aliases);
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

                env.insert(name.clone(), bound_type.clone());
                if let Some(alias) = self.infer_callable_alias(value, callable_aliases) {
                    callable_aliases.insert(name.clone(), alias);
                } else if let Some(namespace) = capability_namespace_alias_for_type(&bound_type) {
                    callable_aliases.insert(name.clone(), namespace.to_string());
                } else {
                    callable_aliases.remove(name);
                }

                if let Some(origin) = infer_value_origin_message(
                    value,
                    &bound_type,
                    callable_aliases,
                    &self.callable_forward_summaries,
                    &self.value_origins,
                ) {
                    self.value_origins.insert(name.clone(), origin);
                } else {
                    self.value_origins.remove(name);
                }
            }
            StmtKind::Return { value } => {
                let return_type = if let Some(value) = value {
                    self.analyze_expr(value, env, used_effects, callable_aliases)
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
                self.analyze_expr(expr, env, used_effects, callable_aliases);
            }
        }
    }

    fn analyze_expr(
        &mut self,
        expr: &Expr,
        env: &mut HashMap<String, Type>,
        used_effects: &mut HashSet<String>,
        callable_aliases: &mut HashMap<String, String>,
    ) -> Type {
        match &expr.kind {
            ExprKind::Identifier(name) => {
                if let Some(ty) = env.get(name) {
                    ty.clone()
                } else if self.catalog.functions.contains_key(name.as_str()) {
                    // Function symbols are valid first-class values for handler-style wiring.
                    Type::Unknown
                } else {
                    self.diagnostics.push(
                        Diagnostic::error("N3003", "unknown identifier", expr.span.clone())
                            .with_note(format!("`{name}` is not defined in this scope")),
                    );
                    Type::Unknown
                }
            }
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
                let inner_type = self.analyze_expr(inner, env, used_effects, callable_aliases);
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
                let left_type = self.analyze_expr(left, env, used_effects, callable_aliases);
                let right_type = self.analyze_expr(right, env, used_effects, callable_aliases);

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
                if let Some(name) = resolve_callable_name(expr, callable_aliases) {
                    if intrinsic_spec_for(name.as_str()).is_some()
                        || self.catalog.functions.contains_key(name.as_str())
                    {
                        return Type::Unknown;
                    }
                }
                let _ = self.analyze_expr(object, env, used_effects, callable_aliases);
                Type::Unknown
            }
            ExprKind::Call { callee, args } => self.analyze_call(
                expr.span.clone(),
                callee,
                args,
                env,
                used_effects,
                callable_aliases,
            ),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_type =
                    self.analyze_expr(condition, env, used_effects, callable_aliases);
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

                let then_type = self.analyze_block(
                    then_branch,
                    env,
                    &Type::Unknown,
                    used_effects,
                    callable_aliases,
                );
                if let Some(else_expr) = else_branch {
                    let else_type =
                        self.analyze_expr(else_expr, env, used_effects, callable_aliases);
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
                let scrutinee_type =
                    self.analyze_expr(scrutinee, env, used_effects, callable_aliases);
                self.analyze_match(
                    expr.span.clone(),
                    &scrutinee_type,
                    arms,
                    env,
                    used_effects,
                    callable_aliases,
                )
            }
            ExprKind::Block(block) => {
                self.analyze_block(block, env, &Type::Unknown, used_effects, callable_aliases)
            }
        }
    }

    fn analyze_call(
        &mut self,
        span: Span,
        callee: &Expr,
        args: &[Expr],
        env: &mut HashMap<String, Type>,
        used_effects: &mut HashSet<String>,
        callable_aliases: &mut HashMap<String, String>,
    ) -> Type {
        let Some(name) = resolve_callable_name(callee, callable_aliases) else {
            self.diagnostics.push(
                Diagnostic::error(
                    "T3104",
                    "unsupported callable expression",
                    callee.span.clone(),
                )
                .with_note("only named functions/constructors are callable in v0.1-lite"),
            );
            for arg in args {
                self.analyze_expr(arg, env, used_effects, callable_aliases);
            }
            return Type::Unknown;
        };

        let arg_types = args
            .iter()
            .map(|arg| self.analyze_expr(arg, env, used_effects, callable_aliases))
            .collect::<Vec<_>>();
        let value_origins = self.value_origins.clone();
        self.enforce_sink_flow_restrictions(
            name.as_str(),
            args,
            &arg_types,
            callable_aliases,
            &value_origins,
        );

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
            if let Some(effect) = intrinsic.effect {
                used_effects.insert(effect.to_string());
            }
            self.enforce_trust_gate_requirements(
                name.as_str(),
                span.clone(),
                args,
                &arg_types,
                callable_aliases,
            );

            if let Some(required_capability) = intrinsic.required_capability {
                let capability_index = capability_arg_index(name.as_str(), args.len());
                match arg_types.get(capability_index) {
                    None => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "E2003",
                                "operation requires capability",
                                span.clone(),
                            )
                            .with_tag("security")
                            .with_tag("capability")
                            .with_note(format!(
                                "`{name}` requires argument {} capability `{required_capability}`",
                                capability_index + 1
                            )),
                        );
                    }
                    Some(actual) if !actual.is_named(required_capability) => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "E2004",
                                "capability type mismatch",
                                args[capability_index].span.clone(),
                            )
                            .with_tag("security")
                            .with_tag("capability")
                            .with_note(format!(
                                "`{name}` expects capability `{required_capability}` at argument {}, got `{}`",
                                capability_index + 1,
                                actual.describe()
                            )),
                        );
                    }
                    Some(_) => {}
                }
            }

            return intrinsic.return_ty.to_type();
        }

        if let Some(constructor) = self.resolve_builtin_constructor(
            name.as_str(),
            span.clone(),
            args,
            env,
            used_effects,
            callable_aliases,
        ) {
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

    fn infer_callable_alias(
        &self,
        expr: &Expr,
        callable_aliases: &HashMap<String, String>,
    ) -> Option<String> {
        if let Some(name) = resolve_callable_name(expr, callable_aliases) {
            if intrinsic_spec_for(name.as_str()).is_some() || is_intrinsic_namespace(name.as_str())
            {
                return Some(name);
            }
            if let Some(summary_target) = self.normalize_callable_forward_target(
                name.clone(),
                &self.callable_forward_summaries,
                false,
            ) {
                return Some(summary_target);
            }
            if self.catalog.functions.contains_key(name.as_str()) {
                return Some(name);
            }
        }

        if let ExprKind::Call { callee, .. } = &expr.kind {
            let callee_name = resolve_callable_name(callee, callable_aliases)?;
            return self.normalize_callable_forward_target(
                callee_name,
                &self.callable_forward_summaries,
                false,
            );
        }
        None
    }

    fn analyze_match(
        &mut self,
        span: Span,
        scrutinee_type: &Type,
        arms: &[MatchArm],
        env: &mut HashMap<String, Type>,
        used_effects: &mut HashSet<String>,
        callable_aliases: &mut HashMap<String, String>,
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

            let value_type =
                self.analyze_expr(&arm.value, &mut arm_env, used_effects, callable_aliases);
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
        callable_aliases: &HashMap<String, String>,
        value_origins: &HashMap<String, String>,
    ) {
        let start_index = sink_user_arg_start_index(callee_name, args.len());
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
                        .with_tag("security")
                        .with_tag("secret")
                        .with_tag("sink")
                        .with_note(format!("sink `{callee_name}` rejects `Secret<_>` values"))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(
                            arg,
                            arg_type,
                            callable_aliases,
                            &self.callable_forward_summaries,
                            value_origins,
                        ))
                        .with_note("use `redact(secret)` or remove the secret from log payload"),
                    );
                } else if arg_type.contains_untrusted() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1002",
                            "untrusted value cannot flow into log sink",
                            arg.span.clone(),
                        )
                        .with_tag("security")
                        .with_tag("taint")
                        .with_tag("sink")
                        .with_note(format!("sink `{callee_name}` requires trusted log values"))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(
                            arg,
                            arg_type,
                            callable_aliases,
                            &self.callable_forward_summaries,
                            value_origins,
                        ))
                        .with_note("validate/sanitize input before constructing log payload"),
                    );
                }
                continue;
            }

            if is_json_sink(callee_name) {
                if !is_json_data_arg(callee_name, index, args.len()) {
                    continue;
                }

                if arg_type.contains_secret() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1004",
                            "secret value cannot be JSON-encoded",
                            arg.span.clone(),
                        )
                        .with_tag("security")
                        .with_tag("secret")
                        .with_tag("sink")
                        .with_note(format!("sink `{callee_name}` rejects `Secret<_>` values"))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(
                            arg,
                            arg_type,
                            callable_aliases,
                            &self.callable_forward_summaries,
                            value_origins,
                        ))
                        .with_note("return a redacted or derived non-secret value"),
                    );
                } else if arg_type.contains_untrusted() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1002",
                            "untrusted value cannot flow into JSON response sink",
                            arg.span.clone(),
                        )
                        .with_tag("security")
                        .with_tag("taint")
                        .with_tag("sink")
                        .with_note(format!("sink `{callee_name}` requires trusted values"))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(
                            arg,
                            arg_type,
                            callable_aliases,
                            &self.callable_forward_summaries,
                            value_origins,
                        ))
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
                        .with_tag("security")
                        .with_tag("secret")
                        .with_tag("sink")
                        .with_note(format!("sink `{callee_name}` rejects `Secret<_>` values"))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(
                            arg,
                            arg_type,
                            callable_aliases,
                            &self.callable_forward_summaries,
                            value_origins,
                        ))
                        .with_note("use non-secret identifiers/values when building SqlQuery"),
                    );
                } else if arg_type.contains_untrusted() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1002",
                            "untrusted value cannot flow into SQL sink",
                            arg.span.clone(),
                        )
                        .with_tag("security")
                        .with_tag("taint")
                        .with_tag("sink")
                        .with_note(format!("sink `{callee_name}` requires trusted SQL inputs"))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(
                            arg,
                            arg_type,
                            callable_aliases,
                            &self.callable_forward_summaries,
                            value_origins,
                        ))
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
                        .with_tag("security")
                        .with_tag("secret")
                        .with_tag("sink")
                        .with_note(format!(
                            "sink `{callee_name}` rejects `Secret<_>` values"
                        ))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(
                            arg,
                            arg_type,
                            callable_aliases,
                            &self.callable_forward_summaries,
                            value_origins,
                        ))
                        .with_note("derive a non-secret `PublicUrl`/`InternalUrl` through URL validation gates"),
                    );
                } else if arg_type.contains_untrusted() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1002",
                            "untrusted value cannot flow into URL sink",
                            arg.span.clone(),
                        )
                        .with_tag("security")
                        .with_tag("taint")
                        .with_tag("sink")
                        .with_note(format!("sink `{callee_name}` requires typed safe URLs"))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(
                            arg,
                            arg_type,
                            callable_aliases,
                            &self.callable_forward_summaries,
                            value_origins,
                        ))
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
                        .with_tag("security")
                        .with_tag("secret")
                        .with_tag("sink")
                        .with_note(format!("sink `{callee_name}` rejects `Secret<_>` values"))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(
                            arg,
                            arg_type,
                            callable_aliases,
                            &self.callable_forward_summaries,
                            value_origins,
                        ))
                        .with_note("use non-secret `PathSafe` values for filesystem operations"),
                    );
                } else if arg_type.contains_untrusted() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1002",
                            "untrusted value cannot flow into filesystem sink",
                            arg.span.clone(),
                        )
                        .with_tag("security")
                        .with_tag("taint")
                        .with_tag("sink")
                        .with_note(format!(
                            "sink `{callee_name}` requires trusted `PathSafe` values"
                        ))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(
                            arg,
                            arg_type,
                            callable_aliases,
                            &self.callable_forward_summaries,
                            value_origins,
                        ))
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
                        .with_tag("security")
                        .with_tag("secret")
                        .with_tag("sink")
                        .with_note(format!("sink `{callee_name}` rejects `Secret<_>` values"))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(
                            arg,
                            arg_type,
                            callable_aliases,
                            &self.callable_forward_summaries,
                            value_origins,
                        ))
                        .with_note(
                            "use redacted/derived values and validated header or cookie builders",
                        ),
                    );
                } else if arg_type.contains_untrusted() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E1002",
                            "untrusted value cannot flow into header/cookie sink",
                            arg.span.clone(),
                        )
                        .with_tag("security")
                        .with_tag("taint")
                        .with_tag("sink")
                        .with_note(format!(
                            "sink `{callee_name}` requires validated header/cookie values"
                        ))
                        .with_note(format!(
                            "argument {} has type `{}`",
                            index + 1,
                            arg_type.describe()
                        ))
                        .with_note(flow_origin_note(
                            arg,
                            arg_type,
                            callable_aliases,
                            &self.callable_forward_summaries,
                            value_origins,
                        ))
                        .with_note(
                            "validate via `validate.headerValue(...)` or typed cookie builders",
                        ),
                    );
                }
            }
        }
    }

    fn enforce_trust_gate_requirements(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
        callable_aliases: &HashMap<String, String>,
    ) {
        self.enforce_http_route_requirements(
            callee_name,
            span.clone(),
            args,
            arg_types,
            callable_aliases,
        );
        self.enforce_http_serve_requirements(callee_name, span.clone(), args, arg_types);
        self.enforce_router_security_bootstrap_requirements(
            callee_name,
            span.clone(),
            args,
            arg_types,
        );

        if is_req_json_gate(callee_name) {
            if args.is_empty() {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "schema gate requires schema argument",
                        span.clone(),
                    )
                    .with_note("`req.json` must be called as `req.json(schema)` in v0.1")
                    .with_note(
                        "this gate converts inbound untrusted payload into trusted typed data",
                    ),
                );
            } else {
                if args.len() != 1 {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E4001",
                            "schema gate expects exactly one schema argument",
                            span.clone(),
                        )
                        .with_tag("security")
                        .with_tag("schema")
                        .with_note("`req.json` call shape is `req.json(schema)`"),
                    );
                }

                let schema_ty = &arg_types[0];
                if schema_ty.is_numeric()
                    || schema_ty.is_bool()
                    || schema_ty.contains_secret()
                    || schema_ty.contains_untrusted()
                {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E4001",
                            "schema gate argument is invalid",
                            args[0].span.clone(),
                        )
                        .with_tag("security")
                        .with_tag("schema")
                        .with_note(format!("found `{}`", schema_ty.describe()))
                        .with_note(
                            "`req.json` expects a schema symbol/descriptor, not numeric/boolean/untrusted/secret data",
                        ),
                    );
                }
            }
        }

        self.enforce_res_text_signature(callee_name, span.clone(), args, arg_types);
        self.enforce_res_html_signature(callee_name, span.clone(), args, arg_types);
        self.enforce_header_cookie_signatures(callee_name, span.clone(), args, arg_types);
        self.enforce_header_builder_signatures(callee_name, span.clone(), args, arg_types);
        self.enforce_cookie_build_signature(callee_name, span.clone(), args, arg_types);
        self.enforce_request_source_signatures(callee_name, span.clone(), args, arg_types);
        self.enforce_path_base_signature(callee_name, span.clone(), args, arg_types);
        self.enforce_sql_q_signature(callee_name, span.clone(), args, arg_types);
        self.enforce_db_query_call_shapes(callee_name, span.clone(), args, arg_types);
        self.enforce_db_tx_call_shape(callee_name, span.clone(), args, arg_types);
        self.enforce_net_sink_call_shapes(callee_name, span.clone(), args, arg_types);
        self.enforce_fs_sink_call_shapes(callee_name, span.clone(), args, arg_types);
        self.enforce_secret_source_call_shapes(callee_name, span.clone(), args, arg_types);
        self.enforce_secret_redact_call_shape(callee_name, span.clone(), args, arg_types);
        self.enforce_secret_reveal_call_shapes(callee_name, span.clone(), args, arg_types);
        self.enforce_auth_helper_call_shapes(callee_name, span.clone(), args, arg_types);
        self.enforce_error_helper_signatures(callee_name, span.clone(), args, arg_types);

        if is_json_sink(callee_name) && self.policy.json.require_schema_for_encode {
            self.enforce_json_encode_signature(callee_name, span.clone(), args, arg_types);
        }

        if is_untrusted_string_gate(callee_name) {
            if args.len() != 1 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "trust gate expects exactly one input argument",
                        span.clone(),
                    )
                    .with_tag("security")
                    .with_tag("schema")
                        .with_note(format!(
                            "`{callee_name}` call shape is `{callee_name}(input)`"
                        )),
                );
                return;
            }

            if !arg_types[0].is_untrusted_string() {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "trust gate expects `Untrusted<String>` input",
                        args[0].span.clone(),
                    )
                    .with_tag("security")
                    .with_tag("schema")
                    .with_note(format!(
                        "`{callee_name}` requires first argument type `Untrusted<String>`"
                    ))
                    .with_note(format!("found `{}`", arg_types[0].describe())),
                );
            }
        }

        if is_path_under_gate(callee_name) {
            if args.len() != 2 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "path gate expects exactly two arguments",
                        span,
                    )
                    .with_tag("security")
                    .with_tag("schema")
                        .with_note(format!(
                            "`{callee_name}` call shape is `{callee_name}(base, input)`"
                        )),
                );
                return;
            }

            if !arg_types[0].is_named("PathSafe") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "path gate expects `PathSafe` base",
                        args[0].span.clone(),
                    )
                    .with_tag("security")
                    .with_tag("schema")
                    .with_note(format!("`{callee_name}` first argument must be `PathSafe`"))
                    .with_note(format!("found `{}`", arg_types[0].describe())),
                );
            }

            if !arg_types[1].is_untrusted_string() {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "path gate expects `Untrusted<String>` input",
                        args[1].span.clone(),
                    )
                    .with_tag("security")
                    .with_tag("schema")
                    .with_note(format!(
                        "`{callee_name}` second argument must be `Untrusted<String>`"
                    ))
                    .with_note(format!("found `{}`", arg_types[1].describe())),
                );
            }
        }
    }

    fn enforce_http_route_requirements(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
        callable_aliases: &HashMap<String, String>,
    ) {
        if !is_http_route_registration(callee_name) {
            return;
        }

        if args.len() != 3 {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "route registration requires `(router, path, handler)` arguments",
                    span,
                )
                .with_note("use `http.get(router, \"/path\", handler)` or `http.post(...)`"),
            );
            return;
        }

        if !arg_types[0].is_named("Router") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "route registration expects `Router` as first argument",
                    args[0].span.clone(),
                )
                .with_note(format!(
                    "`{callee_name}` argument 1 expects `Router`, got `{}`",
                    arg_types[0].describe()
                ))
                .with_note("create router with `http.router()` and pass that value"),
            );
        }

        if !arg_types[1].is_named("String") {
            self.diagnostics.push(
                Diagnostic::error("E4001", "route path must be `String`", args[1].span.clone())
                    .with_note(format!("found `{}`", arg_types[1].describe()))
                    .with_note("use a string path such as `\"/health\"`"),
            );
        }

        let Some(handler_name) = resolve_callable_name(&args[2], callable_aliases) else {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "route handler must reference a declared function symbol",
                    args[2].span.clone(),
                )
                .with_note("pass a function symbol like `health` as the third argument"),
            );
            return;
        };

        let Some(handler_sig) = self.catalog.functions.get(handler_name.as_str()) else {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "route handler must reference a declared function symbol",
                    args[2].span.clone(),
                )
                .with_note(format!("`{handler_name}` is not a declared function")),
            );
            return;
        };

        if !handler_sig.params.is_empty() {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "route handler must not declare parameters in v0 runtime bridge",
                    args[2].span.clone(),
                )
                .with_note(format!(
                    "function `{handler_name}` declares {} parameter(s); route handlers are currently zero-arg",
                    handler_sig.params.len()
                ))
                .with_note(
                    "move request/schema acquisition into the handler body (for example via `req.json(...)`)",
                ),
            );
        }

        if !handler_sig.return_type.is_numeric() {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "route handler must return `Int`/`Int64` in v0 runtime bridge",
                    args[2].span.clone(),
                )
                .with_note(format!(
                    "function `{handler_name}` returns `{}`",
                    handler_sig.return_type.describe()
                ))
                .with_note(
                    "adjust handler return type to `Int` or `Int64` for current runtime bridge compatibility",
                ),
            );
        }

        if !handler_sig.declared_effects.contains("net") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4002",
                    "route handler must declare `net` effect",
                    args[2].span.clone(),
                )
                .with_tag("effects")
                .with_note(format!(
                    "function `{handler_name}` should declare `effects {{ net }}` to be used as an HTTP handler"
                )),
            );
        }
    }

    fn enforce_router_security_bootstrap_requirements(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        let (expected_arity, expected_types, usage_note) = match callee_name {
            "sec.withSecurityHeaders" => (
                2usize,
                vec!["Router", "SecurityHeadersConfig"],
                "use `sec.withSecurityHeaders(router, securityHeadersCfg)`",
            ),
            "cors.withCors" => (
                2usize,
                vec!["Router", "CorsConfig"],
                "use `cors.withCors(router, corsCfg)`",
            ),
            "csrf.withCsrf" => (
                2usize,
                vec!["Router", "CsrfConfig"],
                "use `csrf.withCsrf(router, csrfCfg)`",
            ),
            "auth.withAuth" => (
                2usize,
                vec!["Router", "AuthConfig"],
                "use `auth.withAuth(router, authCfg)`",
            ),
            "sec.defaultHeaders" | "cors.fromPolicy" | "csrf.fromPolicy" | "auth.fromPolicy" => {
                if !args.is_empty() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E4001",
                            "policy/bootstrap constructor takes no arguments",
                            span,
                        )
                        .with_note(format!("`{callee_name}` should be called without arguments")),
                    );
                }
                return;
            }
            _ => return,
        };

        if args.len() != expected_arity {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "security middleware bootstrap call has invalid argument count",
                    span,
                )
                .with_note(format!(
                    "`{callee_name}` expects {expected_arity} arguments, got {}",
                    args.len()
                ))
                .with_note(usage_note),
            );
            return;
        }

        for (index, expected_name) in expected_types.iter().enumerate() {
            if !arg_types[index].is_named(expected_name) {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "security middleware bootstrap argument has invalid type",
                        args[index].span.clone(),
                    )
                    .with_note(format!(
                        "`{callee_name}` argument {} expects `{expected_name}`, got `{}`",
                        index + 1,
                        arg_types[index].describe()
                    ))
                    .with_note(usage_note),
                );
            }
        }
    }

    fn enforce_http_serve_requirements(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if !is_http_serve_call(callee_name) {
            return;
        }

        if args.len() != 2 {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "http.serve requires `(port, router)` arguments",
                    span,
                )
                .with_note("use `http.serve(8080, router)`"),
            );
            return;
        }

        if !arg_types[0].is_numeric() {
            self.diagnostics.push(
                Diagnostic::error("E4001", "http.serve port must be numeric", args[0].span.clone())
                    .with_note(format!("found `{}`", arg_types[0].describe()))
                    .with_note("use `Int`/`Int64` port values such as `8080`"),
            );
        }

        if !arg_types[1].is_named("Router") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "http.serve router argument must be `Router`",
                    args[1].span.clone(),
                )
                .with_note(format!("found `{}`", arg_types[1].describe()))
                .with_note("pass router returned from `http.router()` or middleware chain"),
            );
        }
    }

    fn enforce_json_encode_signature(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        let expected_note = match callee_name {
            "res_ok" | "res.ok" => "strict mode requires `res.ok(status, schema, value)`",
            "res_ok_meta" | "res.okMeta" => {
                "strict mode requires `res.okMeta(status, schema, value, meta)`"
            }
            _ => "strict mode requires `res.json(schema, value)` or `res.json(status, schema, value)`",
        };

        let (status_index, schema_index, value_index) = match callee_name {
            "res_ok" | "res.ok" => {
                if args.len() != 3 {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E4004",
                            "json response encoding has invalid argument count",
                            span,
                        )
                        .with_tag("security")
                        .with_tag("schema")
                        .with_note(expected_note),
                    );
                    return;
                }
                (Some(0), 1, 2)
            }
            "res_ok_meta" | "res.okMeta" => {
                if args.len() != 4 {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E4004",
                            "json response encoding has invalid argument count",
                            span,
                        )
                        .with_tag("security")
                        .with_tag("schema")
                        .with_note(expected_note),
                    );
                    return;
                }
                (Some(0), 1, 2)
            }
            _ => {
                if args.len() < 2 {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E4004",
                            "json response encoding requires explicit schema argument",
                            span,
                        )
                        .with_tag("security")
                        .with_tag("schema")
                        .with_note(expected_note)
                        .with_note(
                            "set `json.require_schema_for_encode = false` in policy to disable strict mode",
                        ),
                    );
                    return;
                }

                if args.len() > 3 {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E4004",
                            "json response encoding has invalid argument count",
                            span,
                        )
                        .with_tag("security")
                        .with_tag("schema")
                        .with_note(expected_note),
                    );
                    return;
                }

                if args.len() == 2 {
                    (None, 0, 1)
                } else {
                    (Some(0), 1, 2)
                }
            }
        };

        if let Some(status_index) = status_index {
            if !arg_types[status_index].is_numeric() {
                let status_note = match callee_name {
                    "res_ok" | "res.ok" => {
                        "use an `Int`/`Int64` status code in `res.ok(status, schema, value)`"
                    }
                    "res_ok_meta" | "res.okMeta" => {
                        "use an `Int`/`Int64` status code in `res.okMeta(status, schema, value, meta)`"
                    }
                    _ => {
                        "use an `Int`/`Int64` status code in `res.json(status, schema, value)`"
                    }
                };

                self.diagnostics.push(
                    Diagnostic::error(
                        "E4004",
                        "json response status must be numeric",
                        args[status_index].span.clone(),
                    )
                    .with_tag("security")
                    .with_tag("schema")
                    .with_note(format!("found `{}`", arg_types[status_index].describe()))
                    .with_note(status_note),
                );
            }
        }

        let schema_ty = &arg_types[schema_index];
        if schema_ty.is_numeric()
            || schema_ty.is_bool()
            || schema_ty.contains_secret()
            || schema_ty.contains_untrusted()
        {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4004",
                    "json response schema argument is invalid",
                    args[schema_index].span.clone(),
                )
                .with_tag("security")
                .with_tag("schema")
                .with_note(format!("found `{}`", schema_ty.describe()))
                .with_note("schema argument should be a schema symbol/descriptor, not numeric/boolean/untrusted/secret data"),
            );
        }

        if let Some(expected_value_ty) = schema_value_type(schema_ty) {
            let actual_value_ty = &arg_types[value_index];
            if !expected_value_ty.compatible_with(actual_value_ty) {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4004",
                        "json response value does not match schema type",
                        args[value_index].span.clone(),
                    )
                    .with_tag("security")
                    .with_tag("schema")
                    .with_note(format!(
                        "schema expects `{}`, got `{}`",
                        expected_value_ty.describe(),
                        actual_value_ty.describe()
                    ))
                    .with_note("adjust response value type or use a matching schema"),
                );
            }
        }
    }

    fn enforce_res_text_signature(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if !is_res_text_call(callee_name) {
            return;
        }

        if args.len() != 2 {
            self.diagnostics.push(
                Diagnostic::error("E4001", "res.text expects `(status, body)` arguments", span)
                    .with_note("use `res.text(200, \"ok\")`"),
            );
            return;
        }

        if !arg_types[0].is_numeric() {
            self.diagnostics.push(
                Diagnostic::error("E4001", "res.text status must be numeric", args[0].span.clone())
                    .with_note(format!("found `{}`", arg_types[0].describe()))
                    .with_note("use `Int`/`Int64` status code values"),
            );
        }

        if !arg_types[1].is_named("String") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "res.text body must be `String`",
                    args[1].span.clone(),
                )
                .with_note(format!("found `{}`", arg_types[1].describe()))
                .with_note("use string body values like `\"ok\"`"),
            );
        }
    }

    fn enforce_res_html_signature(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if !is_res_html_call(callee_name) {
            return;
        }

        if args.len() != 1 {
            self.diagnostics.push(
                Diagnostic::error("E4001", "res.html expects exactly one argument", span)
                    .with_tag("security")
                    .with_tag("sink")
                    .with_note("use `res.html(htmlSafeValue)`"),
            );
            return;
        }

        if !arg_types[0].is_named("HtmlSafe") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "res.html argument must be `HtmlSafe`",
                    args[0].span.clone(),
                )
                .with_tag("security")
                .with_tag("sink")
                .with_note(format!("found `{}`", arg_types[0].describe()))
                .with_note("use `sanitize.html(untrusted)` or another HtmlSafe-producing gate"),
            );
        }
    }

    fn enforce_header_cookie_signatures(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if is_set_header_call(callee_name) {
            if args.len() != 2 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "res.setHeader expects `(name, value)` arguments",
                        span,
                    )
                    .with_tag("security")
                    .with_tag("sink")
                    .with_note("use `res.setHeader(headers.name(...), headers.value(...))`"),
                );
                return;
            }

            if !arg_types[0].is_named("HeaderName") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "res.setHeader name must be `HeaderName`",
                        args[0].span.clone(),
                    )
                    .with_tag("security")
                    .with_tag("sink")
                    .with_note(format!("found `{}`", arg_types[0].describe()))
                    .with_note("construct name with `headers.name(...)`"),
                );
            }

            if !arg_types[1].is_named("HeaderValue") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "res.setHeader value must be `HeaderValue`",
                        args[1].span.clone(),
                    )
                    .with_tag("security")
                    .with_tag("sink")
                    .with_note(format!("found `{}`", arg_types[1].describe()))
                    .with_note("construct value with `headers.value(...)`"),
                );
            }
        }

        if is_add_cookie_call(callee_name) {
            if args.len() != 1 {
                self.diagnostics.push(
                    Diagnostic::error("E4001", "res.addCookie expects exactly one argument", span)
                        .with_tag("security")
                        .with_tag("sink")
                        .with_note("use `res.addCookie(cookieValue)`"),
                );
                return;
            }

            if !arg_types[0].is_named("Cookie") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "res.addCookie argument must be `Cookie`",
                        args[0].span.clone(),
                    )
                    .with_tag("security")
                    .with_tag("sink")
                    .with_note(format!("found `{}`", arg_types[0].describe()))
                    .with_note("construct cookies with `cookie.build(...)`"),
                );
            }
        }
    }

    fn enforce_header_builder_signatures(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if !(is_headers_name_call(callee_name) || is_headers_value_call(callee_name)) {
            return;
        }

        let call_name = if is_headers_name_call(callee_name) {
            "headers.name"
        } else {
            "headers.value"
        };

        if args.len() != 1 {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    format!("{call_name} expects exactly one argument"),
                    span,
                )
                .with_tag("security")
                .with_note(format!("use `{call_name}(\"value\")`")),
            );
            return;
        }

        if !arg_types[0].is_named("String") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    format!("{call_name} argument must be `String`"),
                    args[0].span.clone(),
                )
                .with_tag("security")
                .with_note(format!("found `{}`", arg_types[0].describe()))
                .with_note(format!("use string input for `{call_name}`")),
            );
        }
    }

    fn enforce_cookie_build_signature(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if !is_cookie_build_call(callee_name) {
            return;
        }

        if args.len() != 2 {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "cookie.build expects `(name, value)` arguments",
                    span,
                )
                .with_tag("security")
                .with_note("use `cookie.build(\"name\", \"value\")`"),
            );
            return;
        }

        if !arg_types[0].is_named("String") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "cookie.build name argument must be `String`",
                    args[0].span.clone(),
                )
                .with_tag("security")
                .with_note(format!("found `{}`", arg_types[0].describe()))
                .with_note("use string cookie names"),
            );
        }

        if !arg_types[1].is_named("String") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "cookie.build value argument must be `String`",
                    args[1].span.clone(),
                )
                .with_tag("security")
                .with_note(format!("found `{}`", arg_types[1].describe()))
                .with_note("use string cookie values"),
            );
        }
    }

    fn enforce_request_source_signatures(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if is_req_body_call(callee_name) {
            if args.len() != 2 {
                self.diagnostics.push(
                    Diagnostic::error("E4001", "req.body expects `(ctx, request)` arguments", span)
                        .with_tag("security")
                        .with_tag("schema")
                        .with_note("use `req.body(ctx, request)`"),
                );
                return;
            }

            if !arg_types[0].is_named("Ctx") {
                self.diagnostics.push(
                    Diagnostic::error("E4001", "req.body first argument must be `Ctx`", args[0].span.clone())
                        .with_tag("security")
                        .with_tag("schema")
                        .with_note(format!("found `{}`", arg_types[0].describe()))
                        .with_note("pass handler context as the first argument"),
                );
            }

            if !arg_types[1].is_named("Request") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "req.body second argument must be `Request`",
                        args[1].span.clone(),
                    )
                    .with_tag("security")
                    .with_tag("schema")
                    .with_note(format!("found `{}`", arg_types[1].describe()))
                    .with_note("pass the request value as the second argument"),
                );
            }
            return;
        }

        if !(is_req_query_call(callee_name)
            || is_req_path_param_call(callee_name)
            || is_req_header_call(callee_name))
        {
            return;
        }

        let call_name = if is_req_query_call(callee_name) {
            "req.query"
        } else if is_req_path_param_call(callee_name) {
            "req.pathParam"
        } else {
            "req.header"
        };

        if args.len() != 1 {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    format!("{call_name} expects exactly one argument"),
                    span,
                )
                .with_tag("security")
                .with_tag("schema")
                .with_note(format!("use `{call_name}(\"name\")`")),
            );
            return;
        }

        if !arg_types[0].is_named("String") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    format!("{call_name} argument must be `String`"),
                    args[0].span.clone(),
                )
                .with_tag("security")
                .with_tag("schema")
                .with_note(format!("found `{}`", arg_types[0].describe()))
                .with_note(format!("use string key input for `{call_name}`")),
            );
        }
    }

    fn enforce_path_base_signature(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if !is_path_base_call(callee_name) {
            return;
        }

        if args.len() != 1 {
            self.diagnostics.push(
                Diagnostic::error("E4001", "path.base expects exactly one argument", span)
                    .with_tag("security")
                    .with_note("use `path.base(\"/base/path\")`"),
            );
            return;
        }

        if !arg_types[0].is_named("String") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "path.base argument must be `String`",
                    args[0].span.clone(),
                )
                .with_tag("security")
                .with_note(format!("found `{}`", arg_types[0].describe()))
                .with_note("use string base path values"),
            );
        }
    }

    fn enforce_sql_q_signature(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if !is_sql_q_call(callee_name) {
            return;
        }

        if args.len() != 2 {
            self.diagnostics.push(
                Diagnostic::error("E4001", "sql.q expects `(template, params)` arguments", span)
                    .with_tag("security")
                    .with_tag("schema")
                    .with_note("use `sql.q(\"SELECT ...\", params)`"),
            );
            return;
        }

        if !arg_types[0].is_named("String") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "sql.q template argument must be `String`",
                    args[0].span.clone(),
                )
                .with_tag("security")
                .with_tag("schema")
                .with_note(format!("found `{}`", arg_types[0].describe()))
                .with_note("use SQL template strings such as `\"SELECT ...\"`"),
            );
        }
    }

    fn enforce_db_query_call_shapes(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        let (is_target, valid_shape, note) = if is_db_exec_call(callee_name) {
            (
                true,
                matches!(args.len(), 2 | 3),
                "use `db.exec(capability, query)` or `db.exec(ctx, capability, query)`",
            )
        } else if is_db_exec_tx_call(callee_name) {
            (
                true,
                matches!(args.len(), 2 | 3),
                "use `db.execTx(tx, query)` or `db.execTx(ctx, tx, query)`",
            )
        } else if is_db_query_one_call(callee_name) {
            (
                true,
                matches!(args.len(), 3 | 4),
                "use `db.queryOne(capability, query, rowSchema)` or `db.queryOne(ctx, capability, query, rowSchema)`",
            )
        } else {
            (false, true, "")
        };

        if !is_target || valid_shape {
        } else {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "db sink call has invalid argument shape",
                    span,
                )
                .with_tag("security")
                .with_tag("sink")
                .with_note(note),
            );
            return;
        }

        if is_db_exec_call(callee_name) && args.len() == 3 {
            if !self.enforce_db_sink_context_type(
                callee_name,
                args,
                arg_types,
                "db.exec(ctx, capability, query)",
            ) {
                return;
            }
            self.enforce_db_sink_query_type(callee_name, args, arg_types, 2);
            return;
        }

        if is_db_exec_tx_call(callee_name) && args.len() == 3 {
            if !self.enforce_db_sink_context_type(
                callee_name,
                args,
                arg_types,
                "db.execTx(ctx, tx, query)",
            ) {
                return;
            }
            self.enforce_db_sink_query_type(callee_name, args, arg_types, 2);
            return;
        }

        if is_db_query_one_call(callee_name) && args.len() == 4 {
            if !self.enforce_db_sink_context_type(
                callee_name,
                args,
                arg_types,
                "db.queryOne(ctx, capability, query, rowSchema)",
            ) {
                return;
            }
            self.enforce_db_sink_query_type(callee_name, args, arg_types, 2);
            self.enforce_db_query_one_row_schema_type(args, arg_types, 3);
            return;
        }

        if is_db_exec_call(callee_name) && args.len() == 2 {
            self.enforce_db_sink_query_type(callee_name, args, arg_types, 1);
            return;
        }

        if is_db_exec_tx_call(callee_name) && args.len() == 2 {
            self.enforce_db_sink_query_type(callee_name, args, arg_types, 1);
            return;
        }

        if is_db_query_one_call(callee_name) && args.len() == 3 {
            self.enforce_db_sink_query_type(callee_name, args, arg_types, 1);
            self.enforce_db_query_one_row_schema_type(args, arg_types, 2);
        }
    }

    fn enforce_db_sink_context_type(
        &mut self,
        callee_name: &str,
        args: &[Expr],
        arg_types: &[Type],
        usage: &str,
    ) -> bool {
        let context_type = &arg_types[0];
        if context_type.is_named("Ctx") {
            return true;
        }

        self.diagnostics.push(
            Diagnostic::error(
                "E4001",
                "db sink context argument must be `Ctx`",
                args[0].span.clone(),
            )
            .with_tag("security")
            .with_tag("sink")
            .with_note(format!("found `{}`", context_type.describe()))
            .with_note(format!("use `{usage}` for context-first `{callee_name}` calls")),
        );
        false
    }

    fn enforce_db_sink_query_type(
        &mut self,
        callee_name: &str,
        args: &[Expr],
        arg_types: &[Type],
        query_index: usize,
    ) {
        if arg_types[query_index].contains_untrusted() || arg_types[query_index].contains_secret() {
            return;
        }

        if arg_types[query_index].is_named("SqlQuery") {
            return;
        }

        self.diagnostics.push(
            Diagnostic::error(
                "E4001",
                "db sink query argument must be `SqlQuery`",
                args[query_index].span.clone(),
            )
            .with_tag("security")
            .with_tag("sink")
            .with_note(format!("found `{}`", arg_types[query_index].describe()))
            .with_note(format!(
                "use typed `SqlQuery` values when calling `{callee_name}`"
            )),
        );
    }

    fn enforce_db_query_one_row_schema_type(
        &mut self,
        args: &[Expr],
        arg_types: &[Type],
        row_schema_index: usize,
    ) {
        let row_schema_type = &arg_types[row_schema_index];
        if row_schema_type.contains_untrusted() || row_schema_type.contains_secret() {
            return;
        }

        if row_schema_type.is_numeric() || row_schema_type.is_bool() {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "db.queryOne row schema argument is invalid",
                    args[row_schema_index].span.clone(),
                )
                .with_tag("security")
                .with_tag("schema")
                .with_note(format!("found `{}`", row_schema_type.describe()))
                .with_note(
                    "use `db.queryOne(capability, query, rowSchema)` with a schema descriptor",
                ),
            );
        }
    }

    fn enforce_db_tx_call_shape(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if !is_db_tx_call(callee_name) {
            return;
        }

        if !matches!(args.len(), 1 | 2) {
            self.diagnostics.push(
                Diagnostic::error("E4001", "db.tx call has invalid argument shape", span)
                    .with_tag("security")
                    .with_tag("capability")
                    .with_note("use `db.tx(dbCap)` or `db.tx(ctx, dbCap)`"),
            );
            return;
        }

        if args.len() == 2 && !arg_types[0].is_named("Ctx") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "db.tx context argument must be `Ctx`",
                    args[0].span.clone(),
                )
                .with_tag("security")
                .with_tag("capability")
                .with_note(format!("found `{}`", arg_types[0].describe()))
                .with_note("use `db.tx(ctx, dbCap)` for context-first calls"),
            );
        }
    }

    fn enforce_net_sink_call_shapes(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        let (is_target, valid_shape, note) = if is_net_public_call(callee_name) {
            (
                true,
                matches!(args.len(), 2 | 3),
                "use `httpClient.get(netCap, url)` or `httpClient.get(ctx, netCap, url)`",
            )
        } else if is_net_internal_call(callee_name) {
            (
                true,
                matches!(args.len(), 2 | 3),
                "use `httpClient.getInternal(internalNetCap, url)` or `httpClient.getInternal(ctx, internalNetCap, url)`",
            )
        } else {
            (false, true, "")
        };

        if !is_target {
            return;
        }

        if !valid_shape {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "net sink call has invalid argument shape",
                    span,
                )
                .with_tag("security")
                .with_tag("sink")
                .with_note(note),
            );
            return;
        }

        if args.len() == 3 && !arg_types[0].is_named("Ctx") {
            let usage = if is_net_public_call(callee_name) {
                "httpClient.get(ctx, netCap, url)"
            } else {
                "httpClient.getInternal(ctx, internalNetCap, url)"
            };
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "net sink context argument must be `Ctx`",
                    args[0].span.clone(),
                )
                .with_tag("security")
                .with_tag("sink")
                .with_note(format!("found `{}`", arg_types[0].describe()))
                .with_note(format!(
                    "use `{usage}` for context-first `{callee_name}` calls"
                )),
            );
        }

        let url_index = if args.len() == 3 { 2 } else { 1 };
        let expected_type = if is_net_public_call(callee_name) {
            "PublicUrl"
        } else {
            "InternalUrl"
        };
        if arg_types[url_index].contains_untrusted() || arg_types[url_index].contains_secret() {
            return;
        }
        if !arg_types[url_index].is_named(expected_type) {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    format!("net sink URL argument must be `{expected_type}`"),
                    args[url_index].span.clone(),
                )
                .with_tag("security")
                .with_tag("sink")
                .with_note(format!("found `{}`", arg_types[url_index].describe()))
                .with_note(format!(
                    "use typed `{expected_type}` values when calling `{callee_name}`"
                )),
            );
        }
    }

    fn enforce_fs_sink_call_shapes(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        let (is_target, valid_shape, note) = if is_fs_read_call(callee_name) {
            (
                true,
                matches!(args.len(), 2 | 3),
                "use `fs.read(fsCap, path)` or `fs.read(ctx, fsCap, path)`",
            )
        } else if is_fs_write_call(callee_name) {
            (
                true,
                matches!(args.len(), 3 | 4),
                "use `fs.write(fsCap, path, value)` or `fs.write(ctx, fsCap, path, value)`",
            )
        } else {
            (false, true, "")
        };

        if !is_target {
            return;
        }

        if !valid_shape {
            self.diagnostics.push(
                Diagnostic::error("E4001", "fs sink call has invalid argument shape", span)
                    .with_tag("security")
                    .with_tag("sink")
                    .with_note(note),
            );
            return;
        }

        let needs_context_check = (is_fs_read_call(callee_name) && args.len() == 3)
            || (is_fs_write_call(callee_name) && args.len() == 4);
        if needs_context_check && !arg_types[0].is_named("Ctx") {
            let usage = if is_fs_read_call(callee_name) {
                "fs.read(ctx, fsCap, path)"
            } else {
                "fs.write(ctx, fsCap, path, value)"
            };
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "fs sink context argument must be `Ctx`",
                    args[0].span.clone(),
                )
                .with_tag("security")
                .with_tag("sink")
                .with_note(format!("found `{}`", arg_types[0].describe()))
                .with_note(format!(
                    "use `{usage}` for context-first `{callee_name}` calls"
                )),
            );
        }

        let path_index = if is_fs_read_call(callee_name) {
            if args.len() == 3 { 2 } else { 1 }
        } else if args.len() == 4 {
            2
        } else {
            1
        };
        if arg_types[path_index].contains_untrusted() || arg_types[path_index].contains_secret() {
            return;
        }
        if !arg_types[path_index].is_named("PathSafe") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "fs sink path argument must be `PathSafe`",
                    args[path_index].span.clone(),
                )
                .with_tag("security")
                .with_tag("sink")
                .with_note(format!("found `{}`", arg_types[path_index].describe()))
                .with_note(format!(
                    "use typed `PathSafe` values when calling `{callee_name}`"
                )),
            );
        }
    }

    fn enforce_secret_source_call_shapes(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if !is_secret_get_call(callee_name) {
            return;
        }

        if !matches!(args.len(), 2 | 3) {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "secret source call has invalid argument shape",
                    span,
                )
                .with_tag("security")
                .with_tag("secret")
                .with_note("use `secrets.get(secretsCap, name)` or `secrets.get(ctx, secretsCap, name)`"),
            );
            return;
        }

        if args.len() == 3 && !arg_types[0].is_named("Ctx") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "secret source context argument must be `Ctx`",
                    args[0].span.clone(),
                )
                .with_tag("security")
                .with_tag("secret")
                .with_note(format!("found `{}`", arg_types[0].describe()))
                .with_note("use `secrets.get(ctx, secretsCap, name)` for context-first calls"),
            );
        }

        let name_index = if args.len() == 3 { 2 } else { 1 };
        if !arg_types[name_index].is_named("String") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "secret source name argument must be `String`",
                    args[name_index].span.clone(),
                )
                .with_tag("security")
                .with_tag("secret")
                .with_note(format!("found `{}`", arg_types[name_index].describe()))
                .with_note(
                    "use `secrets.get(secretsCap, name)` or `secrets.get(ctx, secretsCap, name)`",
                ),
            );
        }
    }

    fn enforce_secret_redact_call_shape(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if !is_secret_redact_call(callee_name) {
            return;
        }

        if args.len() != 1 {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "secret redact call expects exactly one argument",
                    span,
                )
                .with_tag("security")
                .with_tag("secret")
                .with_note("use `secrets.redact(secretValue)`"),
            );
            return;
        }

        if !arg_types[0].contains_secret() {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "secret redact argument must be `Secret<_>`",
                    args[0].span.clone(),
                )
                .with_tag("security")
                .with_tag("secret")
                .with_note(format!("found `{}`", arg_types[0].describe()))
                .with_note("use `secrets.redact(secretValue)`"),
            );
        }
    }

    fn enforce_secret_reveal_call_shapes(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if !is_secret_reveal_call(callee_name) {
            return;
        }

        if !matches!(args.len(), 2 | 3) {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "secret reveal call has invalid argument shape",
                    span,
                )
                .with_tag("security")
                .with_tag("secret")
                .with_note(
                    "use `secrets.reveal(secretsCap, secret)` or `secrets.reveal(ctx, secretsCap, secret)`",
                ),
            );
            return;
        }

        if args.len() == 3 && !arg_types[0].is_named("Ctx") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "secret reveal context argument must be `Ctx`",
                    args[0].span.clone(),
                )
                .with_tag("security")
                .with_tag("secret")
                .with_note(format!("found `{}`", arg_types[0].describe()))
                .with_note("use `secrets.reveal(ctx, secretsCap, secret)` for context-first calls"),
            );
        }

        let value_index = if args.len() == 3 { 2 } else { 1 };
        if !arg_types[value_index].contains_secret() {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "secret reveal value argument must be `Secret<_>`",
                    args[value_index].span.clone(),
                )
                .with_tag("security")
                .with_tag("secret")
                .with_note(format!("found `{}`", arg_types[value_index].describe()))
                .with_note(
                    "use `secrets.reveal(secretsCap, secret)` or `secrets.reveal(ctx, secretsCap, secret)`",
                ),
            );
        }
    }

    fn enforce_auth_helper_call_shapes(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if is_auth_require_call(callee_name) {
            if args.len() != 1 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "auth.require expects exactly one argument",
                        span,
                    )
                    .with_tag("security")
                    .with_note("use `auth.require(ctx)`"),
                );
                return;
            }

            if !arg_types[0].is_named("Ctx") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "auth.require argument must be `Ctx`",
                        args[0].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[0].describe()))
                    .with_note("pass request context as argument"),
                );
            }
            return;
        }

        if is_auth_require_role_call(callee_name) {
            if args.len() != 2 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "auth.requireRole expects `(ctx, role)` arguments",
                        span,
                    )
                    .with_tag("security")
                    .with_note("use `auth.requireRole(ctx, \"role\")`"),
                );
                return;
            }

            if !arg_types[0].is_named("Ctx") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "auth.requireRole first argument must be `Ctx`",
                        args[0].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[0].describe()))
                    .with_note("pass request context as first argument"),
                );
            }

            if !arg_types[1].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "auth.requireRole second argument must be `String`",
                        args[1].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[1].describe()))
                    .with_note("pass role name as string"),
                );
            }
        }
    }

    fn enforce_error_helper_signatures(
        &mut self,
        callee_name: &str,
        span: Span,
        args: &[Expr],
        arg_types: &[Type],
    ) {
        if is_err_validation_call(callee_name) {
            if args.len() != 2 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.validation expects `(code, message)` arguments",
                        span,
                    )
                    .with_tag("security")
                    .with_note("use `err.validation(\"CODE\", \"message\")`"),
                );
                return;
            }

            if !arg_types[0].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.validation code argument must be `String`",
                        args[0].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[0].describe()))
                    .with_note("use stable error code strings"),
                );
            }

            if !arg_types[1].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.validation message argument must be `String`",
                        args[1].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[1].describe()))
                    .with_note("use safe validation message strings"),
                );
            }
            return;
        }

        if is_err_auth_call(callee_name) {
            if args.len() != 3 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.auth expects `(code, message, status)` arguments",
                        span,
                    )
                    .with_tag("security")
                    .with_note("use `err.auth(\"CODE\", \"message\", 401)`"),
                );
                return;
            }

            if !arg_types[0].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.auth code argument must be `String`",
                        args[0].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[0].describe()))
                    .with_note("use stable auth error code strings"),
                );
            }

            if !arg_types[1].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.auth message argument must be `String`",
                        args[1].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[1].describe()))
                    .with_note("use safe auth message strings"),
                );
            }

            if !arg_types[2].is_numeric() {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.auth status argument must be numeric",
                        args[2].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[2].describe()))
                    .with_note("use numeric status values such as `401`"),
                );
            }
            return;
        }

        if is_err_not_found_call(callee_name) {
            if args.len() != 2 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.notFound expects `(code, message)` arguments",
                        span,
                    )
                    .with_tag("security")
                    .with_note("use `err.notFound(\"CODE\", \"message\")`"),
                );
                return;
            }

            if !arg_types[0].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.notFound code argument must be `String`",
                        args[0].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[0].describe()))
                    .with_note("use stable not-found error code strings"),
                );
            }

            if !arg_types[1].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.notFound message argument must be `String`",
                        args[1].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[1].describe()))
                    .with_note("use safe not-found message strings"),
                );
            }
            return;
        }

        if is_err_conflict_call(callee_name) {
            if args.len() != 2 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.conflict expects `(code, message)` arguments",
                        span,
                    )
                    .with_tag("security")
                    .with_note("use `err.conflict(\"CODE\", \"message\")`"),
                );
                return;
            }

            if !arg_types[0].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.conflict code argument must be `String`",
                        args[0].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[0].describe()))
                    .with_note("use stable conflict error code strings"),
                );
            }

            if !arg_types[1].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.conflict message argument must be `String`",
                        args[1].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[1].describe()))
                    .with_note("use safe conflict message strings"),
                );
            }
            return;
        }

        if is_err_internal_call(callee_name) {
            if args.len() != 1 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.internal expects exactly one message argument",
                        span,
                    )
                    .with_tag("security")
                    .with_note("use `err.internal(\"message\")`"),
                );
                return;
            }

            if !arg_types[0].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.internal message argument must be `String`",
                        args[0].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[0].describe()))
                    .with_note("use stable safe error messages"),
                );
            }
            return;
        }

        if is_err_with_path_call(callee_name) {
            if args.len() != 2 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.withPath expects `(error, path)` arguments",
                        span,
                    )
                    .with_tag("security")
                    .with_note("use `err.withPath(errorValue, \"$.field\")`"),
                );
                return;
            }

            if !arg_types[1].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.withPath path argument must be `String`",
                        args[1].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[1].describe()))
                    .with_note("use stable path strings such as `\"$.field\"`"),
                );
            }
            return;
        }

        if is_err_with_limit_call(callee_name) {
            if args.len() != 4 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.withLimit expects `(error, name, max, actual)` arguments",
                        span,
                    )
                    .with_tag("security")
                    .with_note("use `err.withLimit(errorValue, \"limit\", max, actual)`"),
                );
                return;
            }

            if !arg_types[1].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.withLimit name argument must be `String`",
                        args[1].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[1].describe()))
                    .with_note("use stable string names for limit descriptors"),
                );
            }

            if !arg_types[2].is_numeric() {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.withLimit max argument must be numeric",
                        args[2].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[2].describe()))
                    .with_note("use `Int`/`Int64` values for max limits"),
                );
            }

            if !arg_types[3].is_numeric() {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.withLimit actual argument must be numeric",
                        args[3].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[3].describe()))
                    .with_note("use `Int`/`Int64` values for actual limits"),
                );
            }
            return;
        }

        if is_err_with_dependency_call(callee_name) {
            if args.len() != 4 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.withDependency expects `(error, name, operation, retryable)` arguments",
                        span,
                    )
                    .with_tag("security")
                    .with_note(
                        "use `err.withDependency(errorValue, \"dependency\", \"op\", retryable)`",
                    ),
                );
                return;
            }

            if !arg_types[1].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.withDependency name argument must be `String`",
                        args[1].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[1].describe()))
                    .with_note("use stable string names for dependencies"),
                );
            }

            if !arg_types[2].is_named("String") {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.withDependency operation argument must be `String`",
                        args[2].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[2].describe()))
                    .with_note("use stable string operation names"),
                );
            }

            if !arg_types[3].is_bool() {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E4001",
                        "err.withDependency retryable argument must be `Bool`",
                        args[3].span.clone(),
                    )
                    .with_tag("security")
                    .with_note(format!("found `{}`", arg_types[3].describe()))
                    .with_note("use boolean retryable flags"),
                );
            }
            return;
        }

        if !is_err_with_detail_call(callee_name) {
            return;
        }

        if args.len() != 3 {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "err.withDetail expects `(error, key, value)` arguments",
                    span,
                )
                .with_tag("security")
                .with_note("use `err.withDetail(errorValue, \"key\", value)`"),
            );
            return;
        }

        if !arg_types[1].is_named("String") {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "err.withDetail key argument must be `String`",
                    args[1].span.clone(),
                )
                .with_tag("security")
                .with_note(format!("found `{}`", arg_types[1].describe()))
                .with_note("use stable string keys for error details"),
            );
        }

        if arg_types[2].contains_secret() {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "err.withDetail value argument cannot be `Secret<_>`",
                    args[2].span.clone(),
                )
                .with_tag("security")
                .with_tag("secret")
                .with_note("redact or derive safe values before attaching error details"),
            );
        } else if arg_types[2].contains_untrusted() {
            self.diagnostics.push(
                Diagnostic::error(
                    "E4001",
                    "err.withDetail value argument cannot be `Untrusted<_>`",
                    args[2].span.clone(),
                )
                .with_tag("security")
                .with_tag("taint")
                .with_note("validate untrusted values before attaching error details"),
            );
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
        callable_aliases: &mut HashMap<String, String>,
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
                let inner = self.analyze_expr(&args[0], env, used_effects, callable_aliases);
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
                    self.analyze_expr(arg, env, used_effects, callable_aliases);
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
                        self.analyze_expr(arg, env, used_effects, callable_aliases);
                    }
                    return Some(Type::result(Type::Unknown, Type::Unknown));
                }
                let ok = self.analyze_expr(&args[0], env, used_effects, callable_aliases);
                Some(Type::result(ok, Type::Unknown))
            }
            "Err" => {
                if args.len() != 1 {
                    self.diagnostics.push(
                        Diagnostic::error("T3103", "constructor argument count mismatch", span)
                            .with_note("`Err` expects exactly one argument"),
                    );
                    for arg in args {
                        self.analyze_expr(arg, env, used_effects, callable_aliases);
                    }
                    return Some(Type::result(Type::Unknown, Type::Unknown));
                }
                let err = self.analyze_expr(&args[0], env, used_effects, callable_aliases);
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
    effect: Option<&'static str>,
    required_capability: Option<&'static str>,
    return_ty: IntrinsicReturnTy,
}

#[derive(Debug, Clone, Copy)]
enum IntrinsicReturnTy {
    Unit,
    Unknown,
    UntrustedString,
    UntrustedBytes,
    Named(&'static str),
}

impl IntrinsicReturnTy {
    fn to_type(self) -> Type {
        match self {
            Self::Unit => Type::Unit,
            Self::Unknown => Type::Unknown,
            Self::UntrustedString => Type::untrusted(Type::named("String")),
            Self::UntrustedBytes => Type::untrusted(Type::named("Bytes")),
            Self::Named(name) => Type::named(name),
        }
    }
}

fn intrinsic_spec_for(name: &str) -> Option<IntrinsicSpec> {
    match name {
        "log" | "log.emit" | "log.info" | "log.warn" | "log.error" => Some(IntrinsicSpec {
            effect: Some("log"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unit,
        }),
        "log_event" | "log.event" | "log_field" | "log.field" | "log_obj" | "log.obj"
        | "log_str" | "log.str" | "log_i64" | "log.i64" | "log_bool" | "log.bool"
        | "log_redacted" | "log.redacted" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("LogValue"),
        }),
        "log_attr_redacted" | "log.attrRedacted" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("LogAttr"),
        }),
        "log_with_attr" | "log.withAttr" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("LogEvent"),
        }),
        "log_with_http" | "log.withHttp" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("LogEvent"),
        }),
        "log_with_error" | "log.withError" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("LogEvent"),
        }),
        "http_router" | "http.router" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("Router"),
        }),
        "http_get_route" | "http.get" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "http_post_route" | "http.post" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "http_serve" | "http.serve" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "withCors" | "cors_with" | "cors.withCors" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("Router"),
        }),
        "withSecurityHeaders" | "sec_with_security_headers" | "sec.withSecurityHeaders" => {
            Some(IntrinsicSpec {
                effect: None,
                required_capability: None,
                return_ty: IntrinsicReturnTy::Named("Router"),
            })
        }
        "withCsrf" | "csrf_with" | "csrf.withCsrf" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("Router"),
        }),
        "withAuth" | "auth_with" | "auth.withAuth" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("Router"),
        }),
        "sec_default_headers" | "sec.defaultHeaders" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("SecurityHeadersConfig"),
        }),
        "sec_csp" | "sec.csp" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("CspPolicy"),
        }),
        "sec_csp_add" | "sec.cspAdd" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("CspPolicy"),
        }),
        "cors_from_policy" | "cors.fromPolicy" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("CorsConfig"),
        }),
        "cors_origin" | "cors.origin" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("Origin"),
        }),
        "csrf_from_policy" | "csrf.fromPolicy" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("CsrfConfig"),
        }),
        "csrf_issue_token" | "csrf.issueToken" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "auth_from_policy" | "auth.fromPolicy" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("AuthConfig"),
        }),
        "auth_require" | "auth.require" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "auth_require_role" | "auth.requireRole" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "err_validation" | "err.validation" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("StdError"),
        }),
        "err_auth" | "err.auth" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("StdError"),
        }),
        "err_not_found" | "err.notFound" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("StdError"),
        }),
        "err_conflict" | "err.conflict" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("StdError"),
        }),
        "err_rate_limit" | "err.rateLimit" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("StdError"),
        }),
        "err_internal" | "err.internal" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("StdError"),
        }),
        "err_with_path" | "err.withPath" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("StdError"),
        }),
        "err_with_detail" | "err.withDetail" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("StdError"),
        }),
        "err_with_limit" | "err.withLimit" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("StdError"),
        }),
        "err_with_dependency" | "err.withDependency" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("StdError"),
        }),
        "err_with_cause" | "err.withCause" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("StdError"),
        }),
        "req_body" | "req.body" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::UntrustedBytes,
        }),
        "req_query" | "req.query" | "req_path_param" | "req.pathParam" | "req_header"
        | "req.header" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::UntrustedString,
        }),
        "req_json" | "req.json" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "json_decode" | "json.decode" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "json_encode" | "json.encode" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("Json"),
        }),
        "res_json" | "res.json" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unit,
        }),
        "res_ok" | "res.ok" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unit,
        }),
        "res_ok_meta" | "res.okMeta" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unit,
        }),
        "res_html" | "res.html" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unit,
        }),
        "res_text" | "res.text" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unit,
        }),
        "set_header" | "res.setHeader" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unit,
        }),
        "set_cookie" | "res.addCookie" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unit,
        }),
        "time_now" | "time.now" => Some(IntrinsicSpec {
            effect: Some("time.now"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "net_call" | "httpClient.get" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: Some("NetCap"),
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "net_internal_call" | "httpClient.getInternal" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: Some("InternalNetCap"),
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "secret_read" | "secrets.get" => Some(IntrinsicSpec {
            effect: Some("secrets.read"),
            required_capability: Some("SecretsCap"),
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "secret_redact" | "secrets.redact" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("String"),
        }),
        "secret_reveal" | "secrets.reveal" => Some(IntrinsicSpec {
            effect: Some("secrets.reveal"),
            required_capability: Some("SecretsCap"),
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "db_read" | "db.queryOne" => Some(IntrinsicSpec {
            effect: Some("db.read"),
            required_capability: Some("DbCap"),
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "db_write" | "db.exec" => Some(IntrinsicSpec {
            effect: Some("db.write"),
            required_capability: Some("DbCap"),
            return_ty: IntrinsicReturnTy::Unit,
        }),
        "db_tx" | "db.tx" => Some(IntrinsicSpec {
            effect: Some("db.tx"),
            required_capability: Some("DbCap"),
            return_ty: IntrinsicReturnTy::Named("TxCap"),
        }),
        "db_exec_tx" | "db.execTx" => Some(IntrinsicSpec {
            effect: Some("db.write"),
            required_capability: Some("TxCap"),
            return_ty: IntrinsicReturnTy::Unit,
        }),
        "sql_q" | "sql.q" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("SqlQuery"),
        }),
        "fs_read" | "fs.read" => Some(IntrinsicSpec {
            effect: Some("fs.read"),
            required_capability: Some("FsCap"),
            return_ty: IntrinsicReturnTy::Unknown,
        }),
        "fs_write" | "fs.write" => Some(IntrinsicSpec {
            effect: Some("fs.write"),
            required_capability: Some("FsCap"),
            return_ty: IntrinsicReturnTy::Unit,
        }),
        "validate_header_value" | "validate.headerValue" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("HeaderValue"),
        }),
        "validate_email" | "validate.email" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("Email"),
        }),
        "validate_uuid" | "validate.uuid" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("Uuid"),
        }),
        "validate_int64" | "validate.int64" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("Int64"),
        }),
        "validate_non_empty" | "validate.nonEmpty" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("String"),
        }),
        "sanitize_html" | "sanitize.html" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("HtmlSafe"),
        }),
        "url_public" | "url.public" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("PublicUrl"),
        }),
        "url_internal" | "url.internal" => Some(IntrinsicSpec {
            effect: Some("net"),
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("InternalUrl"),
        }),
        "path_under" | "path.under" | "validate_path_under" | "validate.pathUnder" => {
            Some(IntrinsicSpec {
                effect: None,
                required_capability: None,
                return_ty: IntrinsicReturnTy::Named("PathSafe"),
            })
        }
        "path_base" | "path.base" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("PathSafe"),
        }),
        "headers_name" | "headers.name" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("HeaderName"),
        }),
        "headers_value" | "headers.value" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("HeaderValue"),
        }),
        "cookie_build" | "cookie.build" => Some(IntrinsicSpec {
            effect: None,
            required_capability: None,
            return_ty: IntrinsicReturnTy::Named("Cookie"),
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

fn resolve_callable_name(
    expr: &Expr,
    callable_aliases: &HashMap<String, String>,
) -> Option<String> {
    let direct = callable_name(expr)?;
    Some(resolve_alias_name(direct, callable_aliases))
}

fn resolve_alias_name(mut name: String, callable_aliases: &HashMap<String, String>) -> String {
    let mut seen = HashSet::new();
    let mut steps = 0usize;
    while seen.insert(name.clone()) {
        steps += 1;
        if steps > 64 {
            break;
        }

        if let Some(next) = callable_aliases.get(name.as_str()) {
            name = next.clone();
            continue;
        }

        let Some((head, tail)) = name.split_once('.') else {
            break;
        };
        let resolved_head = resolve_alias_atom(head, callable_aliases);
        if resolved_head == head {
            break;
        }
        let candidate = format!("{resolved_head}.{tail}");
        if candidate == name || candidate.starts_with(&format!("{name}.")) {
            break;
        }
        name = candidate;
    }
    name
}

fn resolve_summary_name(mut name: String, summaries: &HashMap<String, String>) -> String {
    let mut seen = HashSet::new();
    while seen.insert(name.clone()) {
        let Some(next) = summaries.get(name.as_str()) else {
            break;
        };
        name = next.clone();
    }
    name
}

fn resolve_alias_atom(name: &str, callable_aliases: &HashMap<String, String>) -> String {
    let mut current = name.to_string();
    let mut seen = HashSet::new();
    while seen.insert(current.clone()) {
        let Some(next) = callable_aliases.get(current.as_str()) else {
            break;
        };
        current = next.clone();
    }
    current
}

fn seed_callable_aliases_from_params(
    params: &[crate::ast::Param],
    resolved_param_types: &[Type],
) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    for (index, param) in params.iter().enumerate() {
        let namespace = resolved_param_types
            .get(index)
            .and_then(capability_namespace_alias_for_type)
            .or_else(|| capability_namespace_alias_for_type_expr(&param.ty));
        if let Some(namespace) = namespace {
            aliases.insert(param.name.clone(), namespace.to_string());
        }
    }
    aliases
}

fn seed_callable_aliases_from_param_type_exprs(
    params: &[crate::ast::Param],
) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    for param in params {
        if let Some(namespace) = capability_namespace_alias_for_type_expr(&param.ty) {
            aliases.insert(param.name.clone(), namespace.to_string());
        }
    }
    aliases
}

fn capability_namespace_alias_for_type(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Named { name, args } if args.is_empty() => {
            capability_namespace_alias_for_type_name(name)
        }
        _ => None,
    }
}

fn capability_namespace_alias_for_type_expr(ty: &TypeExpr) -> Option<&'static str> {
    match &ty.kind {
        TypeExprKind::Named { name, args } if args.is_empty() => {
            capability_namespace_alias_for_type_name(name)
        }
        _ => None,
    }
}

fn capability_namespace_alias_for_type_name(name: &str) -> Option<&'static str> {
    match name {
        "DbCap" | "TxCap" => Some("db"),
        "NetCap" | "InternalNetCap" => Some("httpClient"),
        "FsCap" => Some("fs"),
        "SecretsCap" => Some("secrets"),
        _ => None,
    }
}

fn is_intrinsic_namespace(name: &str) -> bool {
    matches!(
        name,
        "db" | "fs" | "sql"
            | "http"
            | "httpClient"
            | "json"
            | "log"
            | "path"
            | "headers"
            | "cookie"
            | "req"
            | "res"
            | "sanitize"
            | "sec"
            | "cors"
            | "csrf"
            | "auth"
            | "secrets"
            | "time"
            | "url"
            | "validate"
    )
}

fn is_log_sink(name: &str) -> bool {
    matches!(
        name,
        "log" | "log.emit" | "log.info" | "log.warn" | "log.error"
    )
}

fn is_json_sink(name: &str) -> bool {
    matches!(
        name,
        "res_json" | "res.json" | "res_ok" | "res.ok" | "res_ok_meta" | "res.okMeta"
    )
}

fn is_sql_sink(name: &str) -> bool {
    matches!(
        name,
        "db_write" | "db.exec" | "db_read" | "db.queryOne" | "db_exec_tx" | "db.execTx"
    )
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

fn sink_user_arg_start_index(name: &str, arg_len: usize) -> usize {
    match name {
        "db_write" | "db.exec" => {
            if arg_len >= 3 {
                2
            } else {
                1
            }
        }
        "db_read" | "db.queryOne" => {
            if arg_len >= 4 {
                2
            } else {
                1
            }
        }
        "db_exec_tx" | "db.execTx" => {
            if arg_len >= 3 {
                2
            } else {
                1
            }
        }
        "net_call" | "httpClient.get" | "net_internal_call" | "httpClient.getInternal" => {
            if arg_len >= 3 {
                2
            } else {
                1
            }
        }
        "fs_read" | "fs.read" => {
            if arg_len >= 3 {
                2
            } else {
                1
            }
        }
        "fs_write" | "fs.write" => {
            if arg_len >= 4 {
                2
            } else {
                1
            }
        }
        "secret_read" | "secrets.get" | "secret_reveal" | "secrets.reveal" => {
            if arg_len >= 3 {
                2
            } else {
                1
            }
        }
        _ => 0,
    }
}

fn capability_arg_index(name: &str, arg_len: usize) -> usize {
    match name {
        "db_write" | "db.exec" => {
            if arg_len >= 3 {
                1
            } else {
                0
            }
        }
        "db_read" | "db.queryOne" => {
            if arg_len >= 4 {
                1
            } else {
                0
            }
        }
        "db_tx" | "db.tx" => {
            if arg_len >= 2 {
                1
            } else {
                0
            }
        }
        "db_exec_tx" | "db.execTx" => {
            if arg_len >= 3 {
                1
            } else {
                0
            }
        }
        "net_call" | "httpClient.get" | "net_internal_call" | "httpClient.getInternal" => {
            if arg_len >= 3 {
                1
            } else {
                0
            }
        }
        "fs_read" | "fs.read" => {
            if arg_len >= 3 {
                1
            } else {
                0
            }
        }
        "fs_write" | "fs.write" => {
            if arg_len >= 4 {
                1
            } else {
                0
            }
        }
        "secret_read" | "secrets.get" | "secret_reveal" | "secrets.reveal" => {
            if arg_len >= 3 {
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}

fn is_req_json_gate(name: &str) -> bool {
    matches!(name, "req_json" | "req.json")
}

fn is_http_route_registration(name: &str) -> bool {
    matches!(
        name,
        "http_get_route" | "http.get" | "http_post_route" | "http.post"
    )
}

fn is_http_serve_call(name: &str) -> bool {
    matches!(name, "http_serve" | "http.serve")
}

fn is_res_text_call(name: &str) -> bool {
    matches!(name, "res_text" | "res.text")
}

fn is_res_html_call(name: &str) -> bool {
    matches!(name, "res_html" | "res.html")
}

fn is_set_header_call(name: &str) -> bool {
    matches!(name, "set_header" | "res.setHeader")
}

fn is_add_cookie_call(name: &str) -> bool {
    matches!(name, "set_cookie" | "res.addCookie")
}

fn is_headers_name_call(name: &str) -> bool {
    matches!(name, "headers_name" | "headers.name")
}

fn is_headers_value_call(name: &str) -> bool {
    matches!(name, "headers_value" | "headers.value")
}

fn is_req_query_call(name: &str) -> bool {
    matches!(name, "req_query" | "req.query")
}

fn is_req_body_call(name: &str) -> bool {
    matches!(name, "req_body" | "req.body")
}

fn is_req_path_param_call(name: &str) -> bool {
    matches!(name, "req_path_param" | "req.pathParam")
}

fn is_req_header_call(name: &str) -> bool {
    matches!(name, "req_header" | "req.header")
}

fn is_sql_q_call(name: &str) -> bool {
    matches!(name, "sql_q" | "sql.q")
}

fn is_cookie_build_call(name: &str) -> bool {
    matches!(name, "cookie_build" | "cookie.build")
}

fn is_path_base_call(name: &str) -> bool {
    matches!(name, "path_base" | "path.base")
}

fn is_db_exec_call(name: &str) -> bool {
    matches!(name, "db_write" | "db.exec")
}

fn is_db_exec_tx_call(name: &str) -> bool {
    matches!(name, "db_exec_tx" | "db.execTx")
}

fn is_db_query_one_call(name: &str) -> bool {
    matches!(name, "db_read" | "db.queryOne")
}

fn is_db_tx_call(name: &str) -> bool {
    matches!(name, "db_tx" | "db.tx")
}

fn is_net_public_call(name: &str) -> bool {
    matches!(name, "net_call" | "httpClient.get")
}

fn is_net_internal_call(name: &str) -> bool {
    matches!(name, "net_internal_call" | "httpClient.getInternal")
}

fn is_fs_read_call(name: &str) -> bool {
    matches!(name, "fs_read" | "fs.read")
}

fn is_fs_write_call(name: &str) -> bool {
    matches!(name, "fs_write" | "fs.write")
}

fn is_secret_get_call(name: &str) -> bool {
    matches!(name, "secret_read" | "secrets.get")
}

fn is_secret_redact_call(name: &str) -> bool {
    matches!(name, "secret_redact" | "secrets.redact")
}

fn is_secret_reveal_call(name: &str) -> bool {
    matches!(name, "secret_reveal" | "secrets.reveal")
}

fn is_auth_require_call(name: &str) -> bool {
    matches!(name, "auth_require" | "auth.require")
}

fn is_auth_require_role_call(name: &str) -> bool {
    matches!(name, "auth_require_role" | "auth.requireRole")
}

fn is_err_with_detail_call(name: &str) -> bool {
    matches!(name, "err_with_detail" | "err.withDetail")
}

fn is_err_with_path_call(name: &str) -> bool {
    matches!(name, "err_with_path" | "err.withPath")
}

fn is_err_with_limit_call(name: &str) -> bool {
    matches!(name, "err_with_limit" | "err.withLimit")
}

fn is_err_with_dependency_call(name: &str) -> bool {
    matches!(name, "err_with_dependency" | "err.withDependency")
}

fn is_err_internal_call(name: &str) -> bool {
    matches!(name, "err_internal" | "err.internal")
}

fn is_err_validation_call(name: &str) -> bool {
    matches!(name, "err_validation" | "err.validation")
}

fn is_err_auth_call(name: &str) -> bool {
    matches!(name, "err_auth" | "err.auth")
}

fn is_err_not_found_call(name: &str) -> bool {
    matches!(name, "err_not_found" | "err.notFound")
}

fn is_err_conflict_call(name: &str) -> bool {
    matches!(name, "err_conflict" | "err.conflict")
}

fn is_json_data_arg(name: &str, index: usize, arg_len: usize) -> bool {
    match name {
        "res_ok_meta" | "res.okMeta" => {
            if arg_len >= 4 {
                index == 2 || index == 3
            } else {
                index + 1 == arg_len
            }
        }
        "res_ok" | "res.ok" => {
            if arg_len >= 3 {
                index == 2
            } else {
                index + 1 == arg_len
            }
        }
        _ => match arg_len {
            0 => false,
            1 => index == 0,
            2 => index == 1,
            _ => index == arg_len - 1,
        },
    }
}

fn schema_value_type(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Named { name, args } if name == "Schema" && args.len() == 1 => args.first(),
        _ => None,
    }
}

fn is_untrusted_string_gate(name: &str) -> bool {
    if is_path_under_gate(name) {
        return false;
    }

    matches!(
        name,
        "validate_header_value"
            | "validate.headerValue"
            | "validate_email"
            | "validate.email"
            | "validate_uuid"
            | "validate.uuid"
            | "validate_int64"
            | "validate.int64"
            | "validate_non_empty"
            | "validate.nonEmpty"
            | "sanitize_html"
            | "sanitize.html"
            | "url_public"
            | "url.public"
            | "url_internal"
            | "url.internal"
            | "cors_origin"
            | "cors.origin"
    ) || name.starts_with("validate.")
        || name.starts_with("validate_")
        || name.starts_with("sanitize.")
        || name.starts_with("sanitize_")
}

fn is_path_under_gate(name: &str) -> bool {
    matches!(
        name,
        "path_under" | "path.under" | "validate_path_under" | "validate.pathUnder"
    )
}

fn flow_origin_note(
    expr: &Expr,
    ty: &Type,
    callable_aliases: &HashMap<String, String>,
    callable_summaries: &HashMap<String, String>,
    value_origins: &HashMap<String, String>,
) -> String {
    match &expr.kind {
        ExprKind::Identifier(name) => value_origins
            .get(name)
            .map(|origin| format!("origin: {origin}"))
            .unwrap_or_else(|| {
                format!(
                    "origin: identifier `{name}` carries type `{}`",
                    ty.describe()
                )
            }),
        ExprKind::Call { callee, .. } => {
            infer_call_origin_message(callee, callable_aliases, callable_summaries)
                .map(|origin| format!("origin: {origin}"))
                .unwrap_or_else(|| {
                    format!(
                        "origin: value comes from call expression of type `{}`",
                        ty.describe()
                    )
                })
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

fn infer_value_origin_message(
    expr: &Expr,
    ty: &Type,
    callable_aliases: &HashMap<String, String>,
    callable_summaries: &HashMap<String, String>,
    value_origins: &HashMap<String, String>,
) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(name) => value_origins.get(name).cloned(),
        ExprKind::Call { callee, .. } => {
            infer_call_origin_message(callee, callable_aliases, callable_summaries)
        }
        ExprKind::Member { object, field } => {
            let base = infer_value_origin_message(
                object,
                ty,
                callable_aliases,
                callable_summaries,
                value_origins,
            )?;
            Some(format!("{base} via member `{field}`"))
        }
        _ => None,
    }
}

fn infer_call_origin_message(
    callee: &Expr,
    callable_aliases: &HashMap<String, String>,
    callable_summaries: &HashMap<String, String>,
) -> Option<String> {
    if let Some(name) = resolve_callable_name(callee, callable_aliases) {
        let chain = callable_summary_chain(name, callable_summaries);
        let resolved = chain
            .last()
            .cloned()
            .unwrap_or_else(|| "<unknown>".to_string());
        if chain.len() > 1 {
            Some(format!(
                "value comes from call `{resolved}(...)` via forwarding chain `{}`",
                chain.join(" -> ")
            ))
        } else {
            Some(format!("value comes from call `{resolved}(...)`"))
        }
    } else if let Some(name) = callable_name(callee) {
        Some(format!("value comes from call `{name}(...)`"))
    } else {
        None
    }
}

fn callable_summary_chain(name: String, summaries: &HashMap<String, String>) -> Vec<String> {
    let mut chain = vec![name.clone()];
    let mut current = name;
    let mut seen = HashSet::new();
    seen.insert(current.clone());
    while let Some(next) = summaries.get(current.as_str()) {
        if !seen.insert(next.clone()) {
            break;
        }
        chain.push(next.clone());
        current = next.clone();
    }
    chain
}

#[derive(Debug, Clone)]
enum PatternCoverage {
    BoolTrue,
    BoolFalse,
    Variant(String),
    CatchAll,
    Other,
}
