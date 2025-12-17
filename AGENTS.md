# Pistachio API Agent Guidelines

## Protobuf Conventions

### Proto3 Syntax Rules (proto3.12+)

`optional` has a strict, technical meaning:
In proto3.12 and later, the `optional` keyword does not mean “this field may
be omitted” in the colloquial sense. All proto3 fields may be omitted on the wire.

Instead, optional explicitly enables **field presence tracking** for scalar fields.

Use `optional`` only when you must distinguish between:

* a field that was never set, and
* a field that was explicitly set to its default value (e.g., "", 0, false).

If this distinction is not required, **do not use** `optional`.

```protobuf
  ```
// Correct — field may be omitted, but presence is not tracked
string display_name = 4;

// Correct — presence is tracked (unset vs explicitly "")
optional string display_name = 5;

// Incorrect — 'optional' used to mean "not required"
optional int32 retry_count = 6;
```
```

Do not use `optional` as a replacement for “not required”:
Proto3 has no concept of required fields. The `optional` keyword is not a
validation or schema constraint.

**Do not use** optional with `repeated` fields:
Repeated fields already have defined presence semantics.

Prefer optional over wrapper types (`google.protobuf.*Value`) when targeting proto3.12+.

### buf.validate and Field Presence

By default, `buf.validate` rules are applied to **all values**, including zero/default
values (empty string, 0, false). This interacts with proto3's lack of presence
tracking for scalar fields.

**Quick rule of thumb:**

| Goal | Solution |
|------|----------|
| Validate only when non-empty | `ignore = IGNORE_IF_ZERO_VALUE` |
| Validate when set, even if empty | Use `optional string` without `IGNORE_IF_ZERO_VALUE` |

**Example: Optional field with format validation**

```protobuf
// Validates format only when provided (empty string skips validation)
string signed_by_kid = 5 [
  (buf.validate.field).ignore = IGNORE_IF_ZERO_VALUE,
  (buf.validate.field).string = {
    len: 70,
    pattern: "^0120[a-fA-F0-9]{64}0[aA]$"
  }
];
```

**Available ignore values:**

| Value | Behavior |
|-------|----------|
| `IGNORE_UNSPECIFIED` | Default: validates all values for fields without presence tracking |
| `IGNORE_IF_ZERO_VALUE` | Skips validation if field equals its zero/default value |
| `IGNORE_ALWAYS` | Always skips validation (useful for temporary disabling) |

### Field Types

* Use `string` for text fields (maps to `TEXT` in SQL)
* Use `google.protobuf.Timestamp` for timestamp fields (maps to `TIMESTAMPTZ` in
  SQL)
* Use `bytes` for binary data

### Naming Conventions

* Use `snake_case` for field names
* Use `PascalCase` for message and enum names
* Use `UPPER_SNAKE_CASE` for enum values

### Documentation

* Every message must have a comment explaining its purpose
* Every field must have a comment explaining what it contains
* Every RPC must document its behavior, required authentication, and possible errors

## File Organization

* Proto files are located in `api/proto/pistachio/v1/`
* Generated code goes in `api/gen/`
