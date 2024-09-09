import cake
import cake/dialect/postgres_dialect
import data/item
import gleam/dynamic
import gleam/list
import gleam/option
import gleam/pgo
import gleeunit
import gleeunit/should

pub fn main() {
  gleeunit.main()
}

fn setup_database() {
  let db =
    pgo.connect(
      pgo.Config(
        ..pgo.default_config(),
        host: "localhost",
        database: "postgres",
        password: option.Some("admin"),
        pool_size: 15,
      ),
    )
  db
}

pub fn read_item_from_db_test() {
  let db = setup_database()

  let return_type =
    dynamic.tuple5(
      dynamic.int,
      dynamic.string,
      dynamic.string,
      dynamic.tuple2(
        dynamic.tuple3(dynamic.int, dynamic.int, dynamic.int),
        dynamic.tuple3(dynamic.int, dynamic.int, dynamic.float),
      ),
      dynamic.tuple2(
        dynamic.tuple3(dynamic.int, dynamic.int, dynamic.int),
        dynamic.tuple3(dynamic.int, dynamic.int, dynamic.float),
      ),
    )

  let assert Ok(response) =
    item.get_items()
    |> postgres_dialect.read_query_to_prepared_statement
    |> cake.get_sql
    |> pgo.execute(db, [], return_type)

  let items = response.rows |> list.map(item.from_dynamic)
  items |> list.length |> should.equal(0)
}
