#[cfg(target_family = "wasm")]
use sapp_jsutils::JsObject;
//#[wasm_bindgen]
//#[no_mangle]
#[cfg(target_family = "wasm")]
extern "C" {
    pub fn get_config() -> sapp_jsutils::JsObject;
}

#[cfg(target_family = "wasm")]
impl MyObject for JsObject {
    fn have_field(&self, field: &str) -> bool {
        self.have_field(field)
    }
    fn is_nil(&self) -> bool {
        self.is_nil()
    }
    fn to_string(&self, string: &mut String) {
        self.to_string(string);
    }
    fn field(&self, field: &str) -> Box<dyn MyObject> {
        Box::new(self.field(field))
    }
}

pub fn get_configuration() -> Box<dyn MyObject> {
    #[cfg(target_family = "wasm")]
    unsafe {
        return Box::new(get_config());
    }

    #[cfg(not(target_family = "wasm"))]
    return Box::new(NothingMuch {});
}

#[cfg(not(target_family = "wasm"))]
struct NothingMuch {}

#[cfg(not(target_family = "wasm"))]
impl MyObject for NothingMuch {
    fn have_field(&self, _field: &str) -> bool {
        false
    }

    fn is_nil(&self) -> bool {
        true
    }

    fn to_string(&self, _: &mut String) {
        panic!("not supported")
    }

    fn field(&self, _: &str) -> Box<dyn MyObject> {
        panic!("not supported")
    }
}

pub trait MyObject {
    fn have_field(&self, field: &str) -> bool;
    fn is_nil(&self) -> bool;
    fn to_string(&self, string: &mut String);
    fn field(&self, field: &str) -> Box<dyn MyObject>;

    fn get_field(&self, field: &str) -> Option<Box<dyn MyObject>> {
        if self.have_field(field) {
            let child = self.field(field);
            if !child.is_nil() {
                return Option::Some(child);
            }
        }

        Option::None
    }
}
