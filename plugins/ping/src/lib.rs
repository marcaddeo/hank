use extism_pdk::*;
use serde::Serialize;

mod test {
    use crate::host_fn;

    #[host_fn]
    extern "ExtismHost" {
        pub fn send_message(content: String) -> String;
    }
}

// #[plugin_fn]
// pub fn ping_handler() {}

#[derive(Serialize)]
struct RegisteredEvents(pub Vec<(String, String)>);

#[plugin_fn]
pub fn init(_: ()) -> FnResult<Json<RegisteredEvents>> {
    let events = vec![("MessageCreate".into(), "ping_handler".into())];

    unsafe {
        let res = test::send_message("Test".into()).unwrap();
        info!("sent message");
        info!("Rust response: {}", res);
    }

    Ok(Json(RegisteredEvents(events)))
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
