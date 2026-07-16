---
name: Competitor Intelligence
description: Gathers and synthesizes information about specific competitors.
author: Nami
parameters:
  - name: competitor_name
    type: string
    description: The name of the competitor to research.
    required: true
  - name: query
    type: string
    description: Optional. A specific aspect of the competitor to focus on (e.g., "product strategy", "latest financials").
    required: false
---

# Competitor Intelligence

This skill is designed to gather and synthesize information about specific competitors. It helps in understanding their strategies, products, market positioning, and overall performance, providing crucial insights for competitive analysis.

## How it works

1.  **Constructs a search query**: It combines the `competitor_name` and optional `query` to create a targeted search string.
2.  **Information Gathering**: It uses `google_search` to find relevant news, company reports, press releases, and industry analyses related to the competitor.
3.  **Analysis & Synthesis (Future Development)**: In future iterations, this skill will integrate `web_fetch` to extract detailed content from search results and `invoke_agent` with the `researcher` specialist to synthesize competitor information into actionable intelligence.
4.  **Output Generation**: The skill will present a summary of competitor insights, key strengths, weaknesses, and strategic implications.

## Example Usage

To research "Google" focusing on "latest AI initiatives":

```python
invoke_agent(specialist="generalist", prompt='''Use the 'competitor_intelligence' skill to research "Google" focusing on "latest AI initiatives".''')
```

## Troubleshooting

*   **No relevant results**: If the initial `google_search` yields no relevant results, try verifying the `competitor_name` or broadening your `query`.
*   **Generic results**: For more specific results, refine your `query` with more detailed keywords.

## Future Enhancements

*   Integration of `web_fetch` for in-depth content extraction from search results.
*   Utilization of `invoke_agent` (researcher specialist) for advanced analysis and synthesis of competitor information.
*   Ability to track competitor activities over time.
*   Automated alerts for significant competitor news or strategy changes.
