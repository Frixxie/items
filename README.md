# Items

This project aims at creating an organization tool for all the stuff you have, be it books, food, camping items or what have you.

## Setup Instructions

### Backend

1. Install Rust: https://www.rust-lang.org/tools/install
2. Clone the repository: `git clone <repository-url>`
3. Navigate to the backend directory: `cd backend`
4. Create a `.env` file in the backend directory with the following content:
   ```
   DATABASE_URL=postgresql://postgres:admin@localhost:5432/postgres
   ```
5. Start the database and MinIO services using Docker Compose: `docker-compose up`
6. Run the backend: `cargo run`

### Frontend

1. Install Deno: https://deno.land/manual/getting_started/installation
2. Navigate to the frontend directory: `cd frontend`
3. Start the frontend: `deno task start`

## Usage Examples

### Backend

- To get the health status of the backend, send a GET request to `/status/health`.
- To get all items, send a GET request to `/api/items`.
- To add a new item, send a POST request to `/api/items` with the following JSON payload:
  ```json
  {
    "name": "Item Name",
    "description": "Item Description",
    "date_origin": "2023-01-01"
  }
  ```

### Frontend

- Open your browser and navigate to `http://localhost:8000` to view the frontend.
