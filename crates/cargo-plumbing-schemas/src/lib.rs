use serde::ser::Serialize;
use serde_json::json;

pub trait Message: Serialize {
    fn reason(&self) -> &str;

    fn to_json_string(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        assert!(json.starts_with("{\""));
        let reason = json!(self.reason());
        format!("{{\"reason\":{},{}", reason, &json[1..])
    }
}

#[test]
fn test_message() {
    #[derive(serde::Serialize)]
    struct TestMessage {
        number: i32,
    }

    impl Message for TestMessage {
        fn reason(&self) -> &str {
            "test-message"
        }
    }

    let t = TestMessage { number: 20 };

    assert_eq!(
        t.to_json_string(),
        r#"{"reason":"test-message","number":20}"#
    );
}
