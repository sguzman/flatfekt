use flatfekt_schema::Scene;
use schemars::schema_for;

fn main() {
  let schema = schema_for!(Scene);
  let json =
    serde_json::to_string_pretty(
      &schema
    )
    .unwrap();
  println!("{}", json);
}
