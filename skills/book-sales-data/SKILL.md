---
name: book-sales-data
description: API for accessing normalized book and transaction data from the database. Use when you need to fetch lists of books, detailed sales transactions, or summary reports. Triggered by requests for book inventory, sales data, or performance summaries.
---

# 📚 Book and Sales Data API Skill

This skill provides Nami with standard methods for querying, sorting, and displaying book sales and transaction telemetry from the system database.

## Instructions

### 1. Data Retrieval
Use `curl` combined with `jq` to fetch and parse JSON data from the localized API endpoints:

*   **Top Selling Authors:** `curl -s http://localhost:8000/summary/author | jq .`
*   **Sales Summary:** `curl -s http://localhost:8000/summary/sales | jq .`
*   **Book Inventory:** `curl -s http://localhost:8000/books | jq .`

### 2. Analytics & Data Processing
*   **Sorting:** Always sort retrieved lists by key metrics such as `Total_Revenue_THB` or `Total_Quantity_Sold` to show the most relevant items first.
*   **Calculations:** Calculate total revenue, top products, or volume trends if requested.

### 3. Beautiful Markdown Presentation
Present data in an elegant, structured table to highlight key parameters:

```markdown
### 🏆 Top-Selling Authors Summary
| Author | Quantity Sold | Total Revenue (THB) |
| :--- | :---: | :--- |
| **John Doe** | 1,420 | ฿426,000 |
| **Jane Smith** | 980 | ฿294,000 |
```

### 4. Robust Error Handling
*   If `curl` fails or endpoints are unreachable, check if the service is running (e.g., `pgrep nami` or checking the port status).
*   Provide readable error messages detailing the status code instead of raw trace logs.