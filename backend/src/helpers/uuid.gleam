import gleam/dynamic as d
import youid/uuid

pub fn from_dynamic(value: d.Dynamic) -> Result(uuid.Uuid, List(d.DecodeError)) {
  let t = d.string(value)
  case t {
    Ok(r) -> {
      let assert Ok(result) = uuid.from_string(r)
      result
    }
    Error(e) -> Error(e)
  }
}
