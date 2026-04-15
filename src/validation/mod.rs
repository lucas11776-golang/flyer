pub mod rules;

use std::{collections::{HashMap, VecDeque}, sync::LazyLock};

use async_std::task::block_on;

use crate::{
    request::{Request, form::{Files, Form}},
    response::Response,
    router::next::Next, utils::Values,
    validation::rules::{
        required,
        string,
        alpha,
        alpha_numeric,
        email,
        min_length,
        max_length,
        numeric,
        url,
        min,
        max,
        confirmed,
        file
    }
};

pub type Rule = dyn Fn(&Form, String, Vec<String>) -> Option<String> + 'static;

static mut RULES: LazyLock<HashMap<String, Box<Rule>>> = LazyLock::new(|| {
    let mut map: HashMap<String, Box<Rule>> = HashMap::new();

    map.insert(String::from("required"), Box::new(|form, field, args| block_on(required(form, field, args))));
    map.insert(String::from("string"), Box::new(|form, field, args| block_on(string(form, field, args))));
    map.insert(String::from("alpha"), Box::new(|form, field, args| block_on(alpha(form, field, args))));
    map.insert(String::from("alpha_numeric"), Box::new(|form, field, args| block_on(alpha_numeric(form, field, args))));
    map.insert(String::from("email"), Box::new(|form, field, args| block_on(email(form, field, args))));
    map.insert(String::from("min_length"), Box::new(|form, field, args| block_on(min_length(form, field, args))));
    map.insert(String::from("max_length"), Box::new(|form, field, args| block_on(max_length(form, field, args))));
    map.insert(String::from("numeric"), Box::new(|form, field, args| block_on(numeric(form, field, args))));
    map.insert(String::from("url"), Box::new(|form, field, args| block_on(url(form, field, args))));
    map.insert(String::from("min"), Box::new(|form, field, args| block_on(min(form, field, args))));
    map.insert(String::from("max"), Box::new(|form, field, args| block_on(max(form, field, args))));
    map.insert(String::from("confirmed"), Box::new(|form, field, args| block_on(confirmed(form, field, args))));
    map.insert(String::from("file"), Box::new(|form, field, args| block_on(file(form, field, args))));

    return map;
});


pub struct Field<'a> {
    pub(crate) name: String,
    pub(crate) rules: Vec<(&'a Box<Rule>, Vec<String>)>
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
            rules: Vec::new()
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

        for rule in rules {
            let mut split = VecDeque::from(rule.split(":").collect::<Vec<&str>>());
            let name = split.pop_front().unwrap();
            let args = split.pop_front().unwrap_or("").split(",").map(|v| String::from(v.trim())).collect::<Vec<String>>();

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
            rules: v
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
        for (rule, args) in &mut field.rules {
            if let Some(error) = rule(form, field.name.clone(), args.to_vec()) {
                return Some(error)
            }
        }

        return None
    }
}