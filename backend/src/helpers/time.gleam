import birl.{type Time}
import gleam/dynamic as d
import gleam/float

fn convert_float_sec_to_int(
  value: #(#(Int, Int, Int), #(Int, Int, Float)),
) -> #(#(Int, Int, Int), #(Int, Int, Int)) {
  case value {
    #(#(y, m, d), #(h, min, sec)) -> #(#(y, m, d), #(h, min, float.round(sec)))
  }
}

pub fn from_dynamic(value: d.Dynamic) -> Result(Time, List(d.DecodeError)) {
  let t =
    d.tuple2(d.tuple3(d.int, d.int, d.int), d.tuple3(d.int, d.int, d.float))(
      value,
    )
  case t {
    Ok(t) ->
      t |> convert_float_sec_to_int |> birl.from_erlang_universal_datetime |> Ok
    Error(e) -> Error(e)
  }
}
