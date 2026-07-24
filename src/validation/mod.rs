use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use futures::future::BoxFuture;

use crate::{
    request::{Request, form::Form},
    response::{HTTP_UNPROCESSABLE_CONTENT, Response},
    routing::next::Next,
    utils::Values,
    validation::rules::*,
};

pub mod rules;

pub type Rule = dyn for<'a> Fn(&'a Form, String, Vec<String>) -> BoxFuture<'a, Option<String>> + Send + Sync + 'static;

pub trait AsyncRule<'a>: Send + Sync {
    type Fut: Future<Output = Option<String>> + Send + 'a;
    fn call(&self, form: &'a Form, field: String, args: Vec<String>) -> Self::Fut;
}

impl<'a, F, Fut> AsyncRule<'a> for F
where
    F: Fn(&'a Form, String, Vec<String>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Option<String>> + Send + 'a,
{
    type Fut = Fut;
    fn call(&self, form: &'a Form, field: String, args: Vec<String>) -> Self::Fut {
        self(form, field, args)
    }
}

pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) rules: Vec<(Arc<Rule>, Vec<String>)>,
    pub(crate) nullable: bool,
}

impl Field {
    pub(crate) fn new(field: impl Into<String>, rules: Vec<(Arc<Rule>, Vec<String>)>, nullable: bool) -> Self {
        return Self {
            name: field.into(),
            rules: rules,
            nullable: nullable,
        };
    }
}

pub struct Validator<'f> {
    pub(crate) form: &'f Form,
    pub(crate) rules: Rules,
    pub(crate) errors: Values,
}

pub struct Rules {
    pub(crate) fields: Vec<Field>,
}

impl Default for Rules {
    fn default() -> Self {
        return Self {
            fields: Vec::new()
        };
    }
}

impl Rules {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(static_mut_refs)]
    pub fn rule(&mut self, field: &str, rules: Vec<&str>) -> &mut Self {
        let mut v = Vec::with_capacity(rules.len());
        let mut is_nullable = false;

        for rule_str in rules {
            let (name, args) = match rule_str.split_once(':') {
                Some((name, args_str)) => {
                    let args = args_str
                        .split(',')
                        .map(|v| v.trim().to_string())
                        .collect::<Vec<String>>();
                    (name, args)
                }
                None => (rule_str, Vec::new()),
            };

            if name == "nullable" {
                is_nullable = true;
                continue;
            }

            let rule_callback = unsafe {
                RULES
                    .get(name)
                    .cloned() // Clones the Arc pointer (very fast)
                    .unwrap_or_else(|| panic!("The rule `{}` does not exist", name))
            };

            v.push((rule_callback, args));
        }

        self.fields.push(Field::new(field, v, is_nullable));

        return self;
    }

    #[allow(static_mut_refs)]
    pub fn add<F>(name: impl Into<String>, callback: F)
    where
        F: for<'a> AsyncRule<'a> + Send + Sync + 'static,
    {
        unsafe {
            RULES.insert(name.into(), Arc::new(move |form, field, args| {
                Box::pin(callback.call(form, field, args))
            }));
        }
    }

    pub async fn handle(self, req: Request, res: Response, next: Next) -> Response {
        return Validator::handle(req, res, next, self).await;
    }
}

impl<'f> Validator<'f> {
    pub fn new(form: &'f Form, rules: Rules) -> Self {
        return Self {
            form: form,
            rules: rules,
            errors: Values::new(),
        };
    }

    pub async fn validate(&mut self) -> bool {
        for field in &mut self.rules.fields {
            if let Some(error) = Self::validate_field(self.form, field).await {
                self.errors.insert(field.name.clone(), error);
            }
        }

        return self.errors.is_empty();
    }

    pub fn errors(&mut self) -> Values {
        return self.errors.clone();
    }

    async fn validate_field(form: &Form, field: &Field) -> Option<String> {
        if field.nullable && crate::validation::rules::is_empty(form, &field.name) {
            return None;
        }

        for (rule, args) in &field.rules {
            if let Some(error) = rule(form, field.name.clone(), args.clone()).await {
                if error.is_empty() {
                    return None;
                }
                return Some(error);
            }
        }

        return None;
    }
}

impl Validator<'_> {
    pub async fn handle(req: Request, res: Response, next: Next, rules: Rules) -> Response {
        let mut validator = Validator::new(&req.form, rules);

        if validator.validate().await {
            return next.handle(req, res);
        }

        if req.is_json() {
            return res
                .status_code(HTTP_UNPROCESSABLE_CONTENT)
                .set_header("Content-Type", "application/json")
                .json(&validator.errors);
        }

        return res
            .with_old(req.form.values.clone())
            .with_errors(validator.errors)
            .back();
    }
}

static mut RULES: LazyLock<HashMap<String, Arc<Rule>>> = LazyLock::new(|| {
    let mut map: HashMap<String, Arc<Rule>> = HashMap::new();

    map.insert(String::from("accepted"), Arc::new(|form, field, args| Box::pin(accepted(form, field, args))));
    map.insert(String::from("accepted_if"), Arc::new(|form, field, args| Box::pin(accepted_if(form, field, args))));
    map.insert(String::from("active_url"), Arc::new(|form, field, args| Box::pin(active_url(form, field, args))));
    map.insert(String::from("after"), Arc::new(|form, field, args| Box::pin(after(form, field, args))));
    map.insert(String::from("after_or_equal"), Arc::new(|form, field, args| Box::pin(after_or_equal(form, field, args))));
    map.insert(String::from("alpha"), Arc::new(|form, field, args| Box::pin(alpha(form, field, args))));
    map.insert(String::from("alpha_dash"), Arc::new(|form, field, args| Box::pin(alpha_dash(form, field, args))));
    map.insert(String::from("alpha_numeric"), Arc::new(|form, field, args| Box::pin(alpha_numeric(form, field, args))));
    map.insert(String::from("alpha_num"), Arc::new(|form, field, args| Box::pin(alpha_numeric(form, field, args))));
    map.insert(String::from("ascii"), Arc::new(|form, field, args| Box::pin(ascii(form, field, args))));
    map.insert(String::from("before"), Arc::new(|form, field, args| Box::pin(before(form, field, args))));
    map.insert(String::from("before_or_equal"), Arc::new(|form, field, args| Box::pin(before_or_equal(form, field, args))));
    map.insert(String::from("between"), Arc::new(|form, field, args| Box::pin(between(form, field, args))));
    map.insert(String::from("boolean"), Arc::new(|form, field, args| Box::pin(boolean(form, field, args))));
    map.insert(String::from("confirmed"), Arc::new(|form, field, args| Box::pin(confirmed(form, field, args))));
    map.insert(String::from("date"), Arc::new(|form, field, args| Box::pin(date(form, field, args))));
    map.insert(String::from("date_equals"), Arc::new(|form, field, args| Box::pin(date_equals(form, field, args))));
    map.insert(String::from("date_format"), Arc::new(|form, field, args| Box::pin(date_format(form, field, args))));
    map.insert(String::from("decimal"), Arc::new(|form, field, args| Box::pin(decimal(form, field, args))));
    map.insert(String::from("declined"), Arc::new(|form, field, args| Box::pin(declined(form, field, args))));
    map.insert(String::from("declined_if"), Arc::new(|form, field, args| Box::pin(declined_if(form, field, args))));
    map.insert(String::from("different"), Arc::new(|form, field, args| Box::pin(different(form, field, args))));
    map.insert(String::from("digits"), Arc::new(|form, field, args| Box::pin(digits(form, field, args))));
    map.insert(String::from("digits_between"), Arc::new(|form, field, args| Box::pin(digits_between(form, field, args))));
    map.insert(String::from("doesnt_start_with"), Arc::new(|form, field, args| Box::pin(doesnt_start_with(form, field, args))));
    map.insert(String::from("doesnt_end_with"), Arc::new(|form, field, args| Box::pin(doesnt_end_with(form, field, args))));
    map.insert(String::from("email"), Arc::new(|form, field, args| Box::pin(email(form, field, args))));
    map.insert(String::from("ends_with"), Arc::new(|form, field, args| Box::pin(ends_with(form, field, args))));
    map.insert(String::from("extensions"), Arc::new(|form, field, args| Box::pin(extensions(form, field, args))));
    map.insert(String::from("file"), Arc::new(|form, field, args| Box::pin(file(form, field, args))));
    map.insert(String::from("filled"), Arc::new(|form, field, args| Box::pin(filled(form, field, args))));
    map.insert(String::from("gt"), Arc::new(|form, field, args| Box::pin(gt(form, field, args))));
    map.insert(String::from("gte"), Arc::new(|form, field, args| Box::pin(gte(form, field, args))));
    map.insert(String::from("hex_color"), Arc::new(|form, field, args| Box::pin(hex_color(form, field, args))));
    map.insert(String::from("image"), Arc::new(|form, field, args| Box::pin(image(form, field, args))));
    map.insert(String::from("in"), Arc::new(|form, field, args| Box::pin(in_rule(form, field, args))));
    map.insert(String::from("integer"), Arc::new(|form, field, args| Box::pin(integer(form, field, args))));
    map.insert(String::from("ip"), Arc::new(|form, field, args| Box::pin(ip(form, field, args))));
    map.insert(String::from("ipv4"), Arc::new(|form, field, args| Box::pin(ipv4(form, field, args))));
    map.insert(String::from("ipv6"), Arc::new(|form, field, args| Box::pin(ipv6(form, field, args))));
    map.insert(String::from("json"), Arc::new(|form, field, args| Box::pin(json(form, field, args))));
    map.insert(String::from("lt"), Arc::new(|form, field, args| Box::pin(lt(form, field, args))));
    map.insert(String::from("lte"), Arc::new(|form, field, args| Box::pin(lte(form, field, args))));
    map.insert(String::from("lowercase"), Arc::new(|form, field, args| Box::pin(lowercase(form, field, args))));
    map.insert(String::from("mac_address"), Arc::new(|form, field, args| Box::pin(mac_address(form, field, args))));
    map.insert(String::from("max"), Arc::new(|form, field, args| Box::pin(max(form, field, args))));
    map.insert(String::from("max_digits"), Arc::new(|form, field, args| Box::pin(max_digits(form, field, args))));
    map.insert(String::from("mimetypes"), Arc::new(|form, field, args| Box::pin(mimetypes(form, field, args))));
    map.insert(String::from("mimes"), Arc::new(|form, field, args| Box::pin(mimes(form, field, args))));
    map.insert(String::from("min"), Arc::new(|form, field, args| Box::pin(min(form, field, args))));
    map.insert(String::from("min_digits"), Arc::new(|form, field, args| Box::pin(min_digits(form, field, args))));
    map.insert(String::from("missing"), Arc::new(|form, field, args| Box::pin(missing(form, field, args))));
    map.insert(String::from("missing_if"), Arc::new(|form, field, args| Box::pin(missing_if(form, field, args))));
    map.insert(String::from("missing_unless"), Arc::new(|form, field, args| Box::pin(missing_unless(form, field, args))));
    map.insert(String::from("multiple_of"), Arc::new(|form, field, args| Box::pin(multiple_of(form, field, args))));
    map.insert(String::from("not_in"), Arc::new(|form, field, args| Box::pin(not_in(form, field, args))));
    map.insert(String::from("not_regex"), Arc::new(|form, field, args| Box::pin(not_regex(form, field, args))));
    map.insert(String::from("numeric"), Arc::new(|form, field, args| Box::pin(numeric(form, field, args))));
    map.insert(String::from("present"), Arc::new(|form, field, args| Box::pin(present(form, field, args))));
    map.insert(String::from("present_if"), Arc::new(|form, field, args| Box::pin(present_if(form, field, args))));
    map.insert(String::from("present_unless"), Arc::new(|form, field, args| Box::pin(present_unless(form, field, args))));
    map.insert(String::from("prohibited"), Arc::new(|form, field, args| Box::pin(prohibited(form, field, args))));
    map.insert(String::from("prohibited_if"), Arc::new(|form, field, args| Box::pin(prohibited_if(form, field, args))));
    map.insert(String::from("prohibited_unless"), Arc::new(|form, field, args| Box::pin(prohibited_unless(form, field, args))));
    map.insert(String::from("prohibited_with"), Arc::new(|form, field, args| Box::pin(prohibited_with(form, field, args))));
    map.insert(String::from("prohibited_with_all"), Arc::new(|form, field, args| Box::pin(prohibited_with_all(form, field, args))));
    map.insert(String::from("regex"), Arc::new(|form, field, args| Box::pin(regex(form, field, args))));
    map.insert(String::from("required"), Arc::new(|form, field, args| Box::pin(required(form, field, args))));
    map.insert(String::from("required_if"), Arc::new(|form, field, args| Box::pin(required_if(form, field, args))));
    map.insert(String::from("required_if_accepted"), Arc::new(|form, field, args| Box::pin(required_if_accepted(form, field, args))));
    map.insert(String::from("required_unless"), Arc::new(|form, field, args| Box::pin(required_unless(form, field, args))));
    map.insert(String::from("required_with"), Arc::new(|form, field, args| Box::pin(required_with(form, field, args))));
    map.insert(String::from("required_with_all"), Arc::new(|form, field, args| Box::pin(required_with_all(form, field, args))));
    map.insert(String::from("required_without"), Arc::new(|form, field, args| Box::pin(required_without(form, field, args))));
    map.insert(String::from("required_without_all"), Arc::new(|form, field, args| Box::pin(required_without_all(form, field, args))));
    map.insert(String::from("same"), Arc::new(|form, field, args| Box::pin(same(form, field, args))));
    map.insert(String::from("size"), Arc::new(|form, field, args| Box::pin(size(form, field, args))));
    map.insert(String::from("starts_with"), Arc::new(|form, field, args| Box::pin(starts_with(form, field, args))));
    map.insert(String::from("string"), Arc::new(|form, field, args| Box::pin(string(form, field, args))));
    map.insert(String::from("uppercase"), Arc::new(|form, field, args| Box::pin(uppercase(form, field, args))));
    map.insert(String::from("url"), Arc::new(|form, field, args| Box::pin(url(form, field, args))));
    map.insert(String::from("ulid"), Arc::new(|form, field, args| Box::pin(ulid(form, field, args))));
    map.insert(String::from("uuid"), Arc::new(|form, field, args| Box::pin(uuid(form, field, args))));

    return map;
});


