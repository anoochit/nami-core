---
name: Market Trend Analysis
description: Identifies and analyzes emerging market trends.
author: Nami
parameters:
  - name: query
    type: string
    description: A description of the market trend to analyze (e.g., "AI in healthcare").
    required: true
  - name: target_market
    type: string
    description: Optional. The specific geographical market to focus on (e.g., "Europe").
    required: false
---

# Market Trend Analysis

This skill is designed to identify and analyze emerging market trends based on a user-provided query. It helps in understanding current and future market dynamics, providing valuable insights for strategic planning.

## How it works

1.  **Search Query Construction**: The skill constructs a comprehensive search query using the provided `query` and optional `target_market`.
2.  **Information Gathering**: It uses `google_search` to find relevant articles, reports, and data on the specified market trends.
3.  **Analysis & Synthesis (Future Development)**: In future iterations, this skill will integrate `web_fetch` to extract detailed content from search results and `invoke_agent` with the `researcher` specialist to synthesize complex information into actionable insights.
4.  **Output Generation**: The skill will present a summary of identified market trends, key findings, and potential implications.

## Example Usage

To analyze "AI in healthcare" in "Europe":

```python
invoke_agent(specialist="generalist", prompt='''Use the 'market_trend_analysis' skill to analyze "AI in healthcare" in "Europe".''')
```

## Troubleshooting

*   **No relevant results**: If the initial `google_search` yields no relevant results, try rephrasing your `query` or broadening your `target_market`.
*   **Generic results**: For more specific results, refine your `query` with more detailed keywords or specify a `target_market`.

## Future Enhancements

*   Integration of `web_fetch` for in-depth content extraction from search results.
*   Utilization of `invoke_agent` (researcher specialist) for advanced analysis and synthesis of information.
*   Ability to save analysis reports in various formats (e.g., PDF, Markdown).
*   Scheduled trend monitoring capabilities.
