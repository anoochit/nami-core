---
name: book-and-sales-data-api
description: API for accessing normalized book and transaction data from the database. Use when you need to fetch lists of books, detailed sales transactions, or summary reports. Triggered by requests for book inventory, sales data, or performance summaries.
---

# Book and Sales Data API

## Overview

The Book and Sales Data API provides access to a database of books and their sales transactions. It allows you to:
- Retrieve book metadata (Title, Author, ISBN).
- Access detailed sales transaction records.
- Generate aggregated summary reports.

## Core Capabilities

### 1. Root
Access the root endpoint.
- **Get root**: `GET /`

### 2. Book Management
Retrieve a list of all books or fetch details for a specific book.
- **Get all books**: `GET /books`
- **Get book details**: `GET /books/{book_id}`

### 3. Sales Transactions
Fetch detailed records of sales, including joined book information for easy reporting.
- **Get all sales**: `GET /sales`

### 4. Summary Reports
Generate aggregated performance data.
- **Summary by Book & Author**: `GET /summary/book_author`
- **Summary by Author**: `GET /summary/author`
- **Summary of Total Revenue by Book**: `GET /summary/book_revenue_total`
- **Monthly Growth Summary**: `GET /summary/growth`

## Reference

See [references/openapi.json](references/openapi.json) for the full OpenAPI specification.

## Example Usage

When a user asks for "Show me the top selling authors," follow these steps:

1. **Fetch the data** using `curl` and parse with `jq` to ensure it is valid JSON:
   ```bash
   curl -s http://localhost:8000/summary/author | jq .
   ```

2. **Process the response**: The result is a JSON array. Sort it by your target metric (e.g., `Total_Revenue_THB` or `Total_Quantity_Sold`) using `jq` if possible, or by presenting the results in a clear table or chart.

3. **Error Handling**: If the API call fails, inform the user with the status code and error message. Ensure the environment has `curl` and `jq` installed.