---
name: Audience Segmentation
description: Researches and defines target audience segments for a given product or service.
author: Nami
parameters:
  - name: target_product_service
    type: string
    description: The product or service for which to define target audience segments.
    required: true
  - name: market_geography
    type: string
    description: Optional. The geographical market to focus on (e.g., "USA", "Europe").
    required: false
---

# Audience Segmentation

This skill is designed to research and define target audience segments for a given product or service. It helps in understanding customer demographics, psychographics, behaviors, and needs, enabling more effective marketing and product development.

## How it works

1.  **Constructs a search query**: It combines the `target_product_service` and optional `market_geography` to create a comprehensive search string.
2.  **Performs a Google Search**: Uses `google_search` to find relevant market research reports, consumer studies, demographic data, and articles on customer behavior.
3.  **Analysis & Synthesis (Future Development)**: In future iterations, this skill will integrate `web_fetch` to extract detailed content from search results and `invoke_agent` with the `researcher` specialist to synthesize audience information into defined segments.
4.  **Output Generation**: The skill will present a summary of identified audience segments, including key characteristics, pain points, and preferences.

## Example Usage

To define audience segments for "eco-friendly cleaning products" in the "USA":

```python
default_api.invoke_agent(specialist="generalist", prompt='''Use the 'audience_segmentation' skill to define target audience segments for "eco-friendly cleaning products" in the "USA".''')
```

## Troubleshooting

*   **No relevant results**: If the initial `google_search` yields no relevant results, try rephrasing your `target_product_service` or broadening your `market_geography`.
*   **Generic results**: For more specific results, refine your `target_product_service` with more detailed keywords.

## Future Enhancements

*   Integration of `web_fetch` for in-depth content extraction from search results.
*   Utilization of `invoke_agent` (researcher specialist) for advanced analysis and definition of audience segments.
*   Creation of audience personas.
*   Integration with internal customer data (if available and permissioned).
