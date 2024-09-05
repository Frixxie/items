import birl.{type Time}
import cake/insert as i
import cake/select as s
import gleam/dynamic as d
import helpers/time

pub type Item {
  Item(
    id: Int,
    name: String,
    description: String,
    date_origin: Time,
    date_recieved: Time,
  )
}

pub fn get_items() {
  s.new()
  |> s.selects([
    s.col("id"),
    s.col("name"),
    s.col("description"),
    s.col("date_origin"),
    s.col("date_recieved"),
  ])
  |> s.from_table("items")
  |> s.to_query
}

pub fn from_dynamic(row) -> Item {
  let assert Ok(item) =
    row
    |> d.from
    |> d.decode5(
      Item,
      d.element(0, d.int),
      d.element(1, d.string),
      d.element(2, d.string),
      d.element(3, time.from_dynamic),
      d.element(4, time.from_dynamic),
    )
  item
}

pub fn insert_item(
  name: String,
  description: String,
  date_origin: Time,
  date_recieved: Time,
) {
  [
    [
      i.string(name),
      i.string(description),
      i.string(date_origin |> birl.to_iso8601()),
      i.string(date_recieved |> birl.to_iso8601()),
    ]
    |> i.row,
  ]
  |> i.from_values(table_name: "items", columns: [
    "name", "description", "date_origin", "date_recieved",
  ])
}
