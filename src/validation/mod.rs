pub mod rules;

use std::{collections::{HashMap, VecDeque}, sync::LazyLock};

use async_std::task::block_on;

use crate::{
    request::{Request, form::{Files, Form}},
    response::Response,
    router::next::Next, utils::Values,
    validation::rules::*
};

pub type Rule = dyn Fn(&Form, String, Vec<String>) -> Option<String> + 'static;

static mut RULES: LazyLock<HashMap<String, Box<Rule>>> = LazyLock::new(|| {
    let mut map: HashMap<String, Box<Rule>> = HashMap::new();

    map.insert(String::from("accepted"), Box::new(|form, field, args| block_on(accepted(form, field, args))));
    map.insert(String::from("accepted_if"), Box::new(|form, field, args| block_on(accepted_if(form, field, args))));
    map.insert(String::from("active_url"), Box::new(|form, field, args| block_on(active_url(form, field, args))));
    map.insert(String::from("after"), Box::new(|form, field, args| block_on(after(form, field, args))));
    map.insert(String::from("after_or_equal"), Box::new(|form, field, args| block_on(after_or_equal(form, field, args))));
    map.insert(String::from("alpha"), Box::new(|form, field, args| block_on(alpha(form, field, args))));
    map.insert(String::from("alpha_dash"), Box::new(|form, field, args| block_on(alpha_dash(form, field, args))));
    map.insert(String::from("alpha_numeric"), Box::new(|form, field, args| block_on(alpha_numeric(form, field, args))));
    map.insert(String::from("alpha_num"), Box::new(|form, field, args| block_on(alpha_numeric(form, field, args))));
    map.insert(String::from("ascii"), Box::new(|form, field, args| block_on(ascii(form, field, args))));
    map.insert(String::from("before"), Box::new(|form, field, args| block_on(before(form, field, args))));
    map.insert(String::from("before_or_equal"), Box::new(|form, field, args| block_on(before_or_equal(form, field, args))));
    map.insert(String::from("between"), Box::new(|form, field, args| block_on(between(form, field, args))));
    map.insert(String::from("boolean"), Box::new(|form, field, args| block_on(boolean(form, field, args))));
    map.insert(String::from("confirmed"), Box::new(|form, field, args| block_on(confirmed(form, field, args))));
    map.insert(String::from("date"), Box::new(|form, field, args| block_on(date(form, field, args))));
    map.insert(String::from("date_equals"), Box::new(|form, field, args| block_on(date_equals(form, field, args))));
    map.insert(String::from("date_format"), Box::new(|form, field, args| block_on(date_format(form, field, args))));
    map.insert(String::from("decimal"), Box::new(|form, field, args| block_on(decimal(form, field, args))));
    map.insert(String::from("declined"), Box::new(|form, field, args| block_on(declined(form, field, args))));
    map.insert(String::from("declined_if"), Box::new(|form, field, args| block_on(declined_if(form, field, args))));
    map.insert(String::from("different"), Box::new(|form, field, args| block_on(different(form, field, args))));
    map.insert(String::from("digits"), Box::new(|form, field, args| block_on(digits(form, field, args))));
    map.insert(String::from("digits_between"), Box::new(|form, field, args| block_on(digits_between(form, field, args))));
    map.insert(String::from("doesnt_start_with"), Box::new(|form, field, args| block_on(doesnt_start_with(form, field, args))));
    map.insert(String::from("doesnt_end_with"), Box::new(|form, field, args| block_on(doesnt_end_with(form, field, args))));
    map.insert(String::from("email"), Box::new(|form, field, args| block_on(email(form, field, args))));
    map.insert(String::from("ends_with"), Box::new(|form, field, args| block_on(ends_with(form, field, args))));
    map.insert(String::from("extensions"), Box::new(|form, field, args| block_on(extensions(form, field, args))));
    map.insert(String::from("file"), Box::new(|form, field, args| block_on(file(form, field, args))));
    map.insert(String::from("filled"), Box::new(|form, field, args| block_on(filled(form, field, args))));
    map.insert(String::from("gt"), Box::new(|form, field, args| block_on(gt(form, field, args))));
    map.insert(String::from("gte"), Box::new(|form, field, args| block_on(gte(form, field, args))));
    map.insert(String::from("hex_color"), Box::new(|form, field, args| block_on(hex_color(form, field, args))));
    map.insert(String::from("image"), Box::new(|form, field, args| block_on(image(form, field, args))));
    map.insert(String::from("in"), Box::new(|form, field, args| block_on(in_rule(form, field, args))));
    map.insert(String::from("integer"), Box::new(|form, field, args| block_on(integer(form, field, args))));
    map.insert(String::from("ip"), Box::new(|form, field, args| block_on(ip(form, field, args))));
    map.insert(String::from("ipv4"), Box::new(|form, field, args| block_on(ipv4(form, field, args))));
    map.insert(String::from("ipv6"), Box::new(|form, field, args| block_on(ipv6(form, field, args))));
    map.insert(String::from("json"), Box::new(|form, field, args| block_on(json(form, field, args))));
    map.insert(String::from("lt"), Box::new(|form, field, args| block_on(lt(form, field, args))));
    map.insert(String::from("lte"), Box::new(|form, field, args| block_on(lte(form, field, args))));
    map.insert(String::from("lowercase"), Box::new(|form, field, args| block_on(lowercase(form, field, args))));
    map.insert(String::from("mac_address"), Box::new(|form, field, args| block_on(mac_address(form, field, args))));
    map.insert(String::from("max"), Box::new(|form, field, args| block_on(max(form, field, args))));
    map.insert(String::from("max_digits"), Box::new(|form, field, args| block_on(max_digits(form, field, args))));
    map.insert(String::from("mimetypes"), Box::new(|form, field, args| block_on(mimetypes(form, field, args))));
    map.insert(String::from("mimes"), Box::new(|form, field, args| block_on(mimes(form, field, args))));
    map.insert(String::from("min"), Box::new(|form, field, args| block_on(min(form, field, args))));
    map.insert(String::from("min_digits"), Box::new(|form, field, args| block_on(min_digits(form, field, args))));
    map.insert(String::from("missing"), Box::new(|form, field, args| block_on(missing(form, field, args))));
    map.insert(String::from("missing_if"), Box::new(|form, field, args| block_on(missing_if(form, field, args))));
    map.insert(String::from("missing_unless"), Box::new(|form, field, args| block_on(missing_unless(form, field, args))));
    map.insert(String::from("multiple_of"), Box::new(|form, field, args| block_on(multiple_of(form, field, args))));
    map.insert(String::from("not_in"), Box::new(|form, field, args| block_on(not_in(form, field, args))));
    map.insert(String::from("not_regex"), Box::new(|form, field, args| block_on(not_regex(form, field, args))));
    map.insert(String::from("numeric"), Box::new(|form, field, args| block_on(numeric(form, field, args))));
    map.insert(String::from("present"), Box::new(|form, field, args| block_on(present(form, field, args))));
    map.insert(String::from("present_if"), Box::new(|form, field, args| block_on(present_if(form, field, args))));
    map.insert(String::from("present_unless"), Box::new(|form, field, args| block_on(present_unless(form, field, args))));
    map.insert(String::from("prohibited"), Box::new(|form, field, args| block_on(prohibited(form, field, args))));
    map.insert(String::from("prohibited_if"), Box::new(|form, field, args| block_on(prohibited_if(form, field, args))));
    map.insert(String::from("prohibited_unless"), Box::new(|form, field, args| block_on(prohibited_unless(form, field, args))));
    map.insert(String::from("prohibited_with"), Box::new(|form, field, args| block_on(prohibited_with(form, field, args))));
    map.insert(String::from("prohibited_with_all"), Box::new(|form, field, args| block_on(prohibited_with_all(form, field, args))));
    map.insert(String::from("regex"), Box::new(|form, field, args| block_on(regex(form, field, args))));
    map.insert(String::from("required"), Box::new(|form, field, args| block_on(required(form, field, args))));
    map.insert(String::from("required_if"), Box::new(|form, field, args| block_on(required_if(form, field, args))));
    map.insert(String::from("required_if_accepted"), Box::new(|form, field, args| block_on(required_if_accepted(form, field, args))));
    map.insert(String::from("required_unless"), Box::new(|form, field, args| block_on(required_unless(form, field, args))));
    map.insert(String::from("required_with"), Box::new(|form, field, args| block_on(required_with(form, field, args))));
    map.insert(String::from("required_with_all"), Box::new(|form, field, args| block_on(required_with_all(form, field, args))));
    map.insert(String::from("required_without"), Box::new(|form, field, args| block_on(required_without(form, field, args))));
    map.insert(String::from("required_without_all"), Box::new(|form, field, args| block_on(required_without_all(form, field, args))));
    map.insert(String::from("same"), Box::new(|form, field, args| block_on(same(form, field, args))));
    map.insert(String::from("size"), Box::new(|form, field, args| block_on(size(form, field, args))));
    map.insert(String::from("starts_with"), Box::new(|form, field, args| block_on(starts_with(form, field, args))));
    map.insert(String::from("string"), Box::new(|form, field, args| block_on(string(form, field, args))));
    map.insert(String::from("uppercase"), Box::new(|form, field, args| block_on(uppercase(form, field, args))));
    map.insert(String::from("url"), Box::new(|form, field, args| block_on(url(form, field, args))));
    map.insert(String::from("ulid"), Box::new(|form, field, args| block_on(ulid(form, field, args))));
    map.insert(String::from("uuid"), Box::new(|form, field, args| block_on(uuid(form, field, args))));

    return map;
});


pub struct Field<'a> {
    pub(crate) name: String,
    pub(crate) rules: Vec<(&'a Box<Rule>, Vec<String>)>,
    pub(crate) nullable: bool,
}


#[allow(dead_code)]
pub struct Validator<'f> {
    pub(crate) form: &'f Form,
    pub(crate) rules: Rules<'f>,
    pub(crate) errors: Values,
    pub(crate) validated: Form,
}

impl <'f>Field<'f> {
    pub(crate) fn new(field: &str) -> Field<'f> {
        return Self {
            name: field.to_string(),
            rules: Vec::new(),
            nullable: false,
        }
    }

    // pub fn add<R>(&mut self, callback: R, args: Vec<&str>) -> &mut Field<'f>
    // where
    //     R: for<'a> AsyncFn(&'f Form, String, Vec<String>) -> Option<String> + Send + Sync + 'static
    // {
    //     // self.rules.push((
    //     //     Box::new(move |form, field, args| block_on(callback(form, field, args))),
    //     //     args.iter().map(|v| v.to_string()).collect()
    //     // ));

    //     return self;
    // }
}

pub struct Rules<'r> {
    pub(crate) fields: Vec<Field<'r>>
}

impl <'r>Rules<'r> {
    pub fn new() -> Rules<'r> {
        return Self {
            fields: Vec::new(),
        }
    }

    #[deprecated]
    pub fn field(&mut self, field: &str) -> &mut Field<'r> {
        let idx = self.fields.len();

        self.fields.push(Field::new(field));

        return &mut self.fields[idx];
    }

    #[allow(static_mut_refs)]
    pub fn rule(&mut self, field: &str, rules: Vec<&str>) -> &mut Self {
        let mut v = Vec::new();
        let mut is_nullable = false;

        for rule in rules {
            let mut split = VecDeque::from(rule.split(":").collect::<Vec<&str>>());
            let name = split.pop_front().unwrap();
            let args = split.pop_front().unwrap_or("").split(",").map(|v| String::from(v.trim())).collect::<Vec<String>>();

            if name == "nullable" {
                is_nullable = true;
                continue;
            }

            let rule_callback = unsafe {
                match RULES.get(name) {
                    Some(rule) => rule,
                    None => panic!("The rule `{}` does not exist", name),
                }
            };

            v.push((rule_callback, args));
        };

        self.fields.push(Field {
            name: String::from(field),
            rules: v,
            nullable: is_nullable,
        });

        return self;
    }

    #[allow(static_mut_refs)]
    pub fn add<R>(name: &str, callback: R)
    where
        R: for<'c> AsyncFn(&'c Form, String, Vec<String>) -> Option<String> + Send + Sync + 'static
    {
        unsafe {
            RULES.insert(String::from(name), Box::new(move |form, field, args| block_on(callback(form, field, args))));
        };
    }

    pub fn handle(self, req: &'r mut Request, res: &'r mut Response, next: &'r mut Next) -> &'r mut Response {
        return block_on(Validator::handle(req, res, next, self));
    }

}

impl <'a>Validator<'a> {
    pub fn new(form: &'a Form, rules: Rules<'a>) -> Validator<'a> {
        return Self {
            form: form,
            rules: rules,
            errors: Values::new(),
            validated: Form::new(Values::new(), Files::new())
        }
    }

    pub async fn validate(&mut self) -> bool {
        for field in &mut self.rules.fields {
            if let Some(error) = Self::validate_field(self.form, field) {
                self.errors.insert(field.name.to_string(), error);
            }
        }

        return self.errors.len() == 0;
    }

    pub async fn handle(req: &'a mut Request, res: &'a mut Response, next: &'a mut Next, rules: Rules<'a>) -> &'a mut Response {
        let mut validator = Self::new(&req.form, rules);

        if !validator.validate().await {
            return res.with_old(req.form.values.clone()).with_errors(validator.errors).back();
        }

        return next.handle(res);
    }

    pub fn errors(&mut self) -> Values {
        return self.errors.clone();
    }

    fn validate_field(form: &Form, field: &mut Field) -> Option<String> {
        if field.nullable && crate::validation::rules::is_empty(form, &field.name) {
            return None;
        }
        for (rule, args) in &mut field.rules {
            if let Some(error) = rule(form, field.name.clone(), args.to_vec()) {
                return Some(error)
            }
        }

        return None
    }
}