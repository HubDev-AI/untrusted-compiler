# 56 Security Diagnostics Taxonomy

This chapter defines stable compiler diagnostics for security/effect/type/policy behavior.

## Error code conventions

- Prefix `E` for errors, `W` for warnings.
- Category ranges:
  - `E1xxx`: trust/secret flow
  - `E2xxx`: effects/capabilities
  - `E3xxx`: core typing/shape
  - `E4xxx`: schema
  - `E5xxx`: SQL/HTML/header template/sink rules
  - `E6xxx`: policy/hardening mode

Every diagnostic should include:
1. primary span
2. origin trace
3. sink/API expectation
4. fix-it guidance

## E1xxx — trust and secret flow

- `E1001` UntrustedToTrusted
- `E1002` UntrustedToSink
- `E1003` SecretLeakToLog
- `E1004` SecretLeakToJson
- `E1005` SecretLeakToFormat
- `E1006` SecretRevealForbidden

## E2xxx — effects and capabilities

- `E2001` MissingEffectDeclaration
- `E2002` ForbiddenEffectByPolicy
- `E2003` MissingCapability
- `E2004` CapabilityMismatch
- `E2005` EffectOnFunctionFieldMissing

## E3xxx — core types and structural shapes

- `E3001` FieldNotFound
- `E3002` FieldTypeMismatch
- `E3003` ShapeNotSatisfied
- `E3004` ShapeIntersectionConflict
- `E3005` OptionMisuse

## E4xxx — schema system

- `E4001` SchemaFieldMissingValidator
- `E4002` SchemaValidatorTypeMismatch
- `E4003` SchemaTypeNotEncodable
- `E4004` SchemaContainsSecret
- `E4005` JsonBudgetMissing

## E5xxx — templates and sink typing

- `E5001` SqlParamNotAllowed
- `E5002` SqlMissingDbCapOrEffect
- `E5003` HtmlInterpolationNotAllowed
- `E5004` HeaderValueInvalidConstruction

## E6xxx — policy and hardened mode

- `E6001` InternalNetForbidden
- `E6002` RedirectsForbidden
- `E6003` RawLoggingForbidden
- `E6004` EncodingWithoutSchemaForbidden

## Warnings (optional)

- `W1101` LoggingUntrusted
- `W1201` UnboundedQuery
- `W1301` BroadCapabilityInHandler

## Origin tracing requirement

For security/effect errors, diagnostics should show:
- where value/effect originated
- where it failed (sink/API)
- canonical fix path (schema, validator, sanitizer, capability, effect declaration)
