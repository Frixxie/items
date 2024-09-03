import birl.{type Time}


pub type Item {
  Item(
    id: Int,
    name: String,
    description: String,
    date_origin: Time,
    date_recieved: Time,
  )
}


