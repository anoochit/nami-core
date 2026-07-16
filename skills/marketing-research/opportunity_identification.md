---
name: opportunity_identification
description: Identifies new market niches or product opportunities within a specified industry.
parameters:
  industry:
    type: string
    description: The industry to research for opportunities.
    required: true
  query:
    type: string
    description: An optional query to focus the search on a specific area (e.g., "innovation gaps", "untapped markets").
    required: false
example:
  - prompt: Identify innovation gaps in vertical farming within the sustainable agriculture industry.
    code: |
      print(default_api.opportunity_identification(industry="sustainable agriculture", query="innovation gaps in vertical farming"))
---

# Opportunity Identification Skill

This skill is designed to identify new market niches, product opportunities, or innovation gaps within a specified industry. It helps businesses discover untapped potential and make informed decisions about new ventures.

## Quick-Start Instructions
To use this skill, invoke it with the `industry` you want to research for opportunities. You can optionally provide a `query` to focus on a specific area or question (e.g., "untapped markets", "innovation gaps").
