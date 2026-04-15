# Validation

Flyer provides a powerful and flexible validation system inspired by Laravel. It allows you to validate incoming request data using a simple, string-based rule syntax.

## Usage

### Basic Validation

You can manually create a `Validator` to check form data.

```rust
use flyer::validation::{Validator, Rules};

// Inside a handler
let mut rules = Rules::new();
rules.rule("email", vec!["required", "email"]);
rules.rule("age", vec!["required", "numeric", "min:18"]);

let mut validator = Validator::new(&req.form, rules);

if validator.validate().await {
    // Data is valid
} else {
    // Get validation errors
    let errors = validator.errors();
}
```

### Middleware Validation

You can also use validation as a middleware that automatically redirects back with errors if validation fails.

```rust
use flyer::validation::Rules;

let mut rules = Rules::new();
rules.rule("username", vec!["required", "alpha_num", "between:3,20"]);

rules.handle(req, res, next)
```

## Available Rules

Rules can be passed as strings. Some rules accept arguments separated by a colon (`:`), and multiple arguments are separated by commas (`,`).

### Booleans

- **accepted**: The field must be "yes", "on", 1, "true", or true.
- **accepted_if:anotherfield,value**: The field must be accepted if `anotherfield` is equal to `value`.
- **boolean**: The field must be a boolean representation ("true", "false", 1, 0, "on", "off", "yes", "no").
- **declined**: The field must be "no", "off", 0, "false", or false.
- **declined_if:anotherfield,value**: The field must be declined if `anotherfield` is equal to `value`.

### Strings

- **active_url**: The field must be a valid URL and the host must be reachable.
- **alpha**: The field must be entirely alphabetic characters.
- **alpha_dash**: The field may have alpha-numeric characters, dashes, and underscores.
- **alpha_num**: The field must be entirely alpha-numeric characters.
- **ascii**: The field must be entirely 7-bit ASCII characters.
- **confirmed**: The field must have a matching field of `{field}_confirmation`.
- **different:field**: The field must have a different value than the given field.
- **doesnt_start_with:foo,bar,...**: The field must not start with any of the given values.
- **doesnt_end_with:foo,bar,...**: The field must not end with any of the given values.
- **email**: The field must be formatted as an email address.
- **ends_with:foo,bar,...**: The field must end with one of the given values.
- **hex_color**: The field must be a valid hexadecimal color code (e.g., #000 or #FFFFFF).
- **in:foo,bar,...**: The field must be included in the given list of values.
- **ip**: The field must be a valid IP address.
- **ipv4**: The field must be a valid IPv4 address.
- **ipv6**: The field must be a valid IPv6 address.
- **json**: The field must be a valid JSON string.
- **lowercase**: The field must be lowercase.
- **mac_address**: The field must be a valid MAC address.
- **not_in:foo,bar,...**: The field must not be included in the given list of values.
- **regex:pattern**: The field must match the given regular expression.
- **not_regex:pattern**: The field must not match the given regular expression.
- **same:field**: The field must match the given field.
- **starts_with:foo,bar,...**: The field must start with one of the given values.
- **string**: The field must be a string.
- **uppercase**: The field must be uppercase.
- **url**: The field must be a valid URL.
- **ulid**: The field must be a valid ULID.
- **uuid**: The field must be a valid UUID.

### Numbers

- **between:min,max**: 
  - For numbers: must be between `min` and `max`.
  - For strings: length must be between `min` and `max` characters.
  - For files: size must be between `min` and `max` kilobytes.
- **decimal:min[,max]**: The field must be numeric and have between `min` and `max` decimal places.
- **digits:value**: The field must be numeric and have an exact length of `value`.
- **digits_between:min,max**: The field must be numeric and have a length between `min` and `max`.
- **gt:field_or_value**: The field must be greater than another field or a literal value.
- **gte:field_or_value**: The field must be greater than or equal to another field or a literal value.
- **integer**: The field must be an integer.
- **lt:field_or_value**: The field must be less than another field or a literal value.
- **lte:field_or_value**: The field must be less than or equal to another field or a literal value.
- **max:value**:
  - For numbers: must not be greater than `value`.
  - For strings: must not be longer than `value` characters.
  - For files: must not be larger than `value` kilobytes.
- **min:value**:
  - For numbers: must be at least `value`.
  - For strings: must be at least `value` characters.
  - For files: must be at least `value` kilobytes.
- **max_digits:value**: The field must have a maximum length of `value` digits.
- **min_digits:value**: The field must have a minimum length of `value` digits.
- **multiple_of:value**: The field must be a multiple of `value`.
- **numeric**: The field must be a number.

### Dates

Dates support multiple formats: `YYYY-MM-DD HH:MM:SS`, `YYYY-MM-DD`, `DD-MM-YYYY`, `MM/DD/YYYY`.

- **after:date_or_field**: The field must be a date after the given date or field.
- **after_or_equal:date_or_field**: The field must be a date after or equal to the given date or field.
- **before:date_or_field**: The field must be a date before the given date or field.
- **before_or_equal:date_or_field**: The field must be a date before or equal to the given date or field.
- **date**: The field must be a valid date.
- **date_equals:date_or_field**: The field must be equal to the given date or field.
- **date_format:format**: The field must match the given chrono format (e.g., `%Y-%m-%d`).

### Files

- **file**: The field must be a successfully uploaded file.
- **image**: The file must be an image (jpg, jpeg, png, gif, bmp, svg, webp).
- **mimetypes:type1,type2,...**: The file must match one of the given MIME types.
- **mimes:ext1,ext2,...**: The file must have one of the given extensions.
- **extensions:ext1,ext2,...**: Alias for `mimes`.

### Utilities

- **required**: The field must be present and not empty.
- **required_if:anotherfield,value**: Required if `anotherfield` equals `value`.
- **required_if_accepted:anotherfield**: Required if `anotherfield` is accepted.
- **required_unless:anotherfield,value**: Required unless `anotherfield` equals `value`.
- **required_with:foo,bar,...**: Required if *any* of the other fields are present.
- **required_with_all:foo,bar,...**: Required if *all* of the other fields are present.
- **required_without:foo,bar,...**: Required if *any* of the other fields are missing.
- **required_without_all:foo,bar,...**: Required if *all* of the other fields are missing.
- **filled**: If the field is present, it must not be empty.
- **missing**: The field must not be present.
- **missing_if:anotherfield,value**: Missing if `anotherfield` equals `value`.
- **missing_unless:anotherfield,value**: Missing unless `anotherfield` equals `value`.
- **present**: The field must be present (can be empty).
- **present_if:anotherfield,value**: Present if `anotherfield` equals `value`.
- **present_unless:anotherfield,value**: Present unless `anotherfield` equals `value`.
- **prohibited**: The field must be missing or empty.
- **prohibited_if:anotherfield,value**: Prohibited if `anotherfield` equals `value`.
- **prohibited_unless:anotherfield,value**: Prohibited unless `anotherfield` equals `value`.
- **prohibited_with:foo,bar,...**: Prohibited if *any* of the other fields are present.
- **prohibited_with_all:foo,bar,...**: Prohibited if *all* of the other fields are present.
- **size:value**:
  - For numbers: must equal `value`.
  - For strings: must be exactly `value` characters.
  - For files: must be exactly `value` kilobytes.

## Custom Rules

You can add custom rules globally using `Rules::add`.

```rust
use flyer::validation::Rules;

Rules::add("custom_rule", async |form, field, args| {
    Some("Error message".to_string())
});
```
