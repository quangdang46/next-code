# Mermaid Sample — Advanced

## Pie Chart

```mermaid
pie title Stack Overflow Survey 2026
    "JavaScript" : 32
    "TypeScript" : 28
    "Python" : 24
    "Rust" : 10
    "Go" : 6
```

## State Diagram

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Loading : fetch data
    Loading --> Success : response OK
    Loading --> Error : timeout/error
    Success --> Idle : refresh
    Error --> Loading : retry
    Error --> Idle : dismiss
```

## Architecture (C4-style)

```mermaid
flowchart LR
    subgraph Client
        UI[Next.js UI]
    end
    subgraph Edge
        CDN[Cloudflare CDN]
        LB[Load Balancer]
    end
    subgraph Backend
        API[REST API]
        WS[WebSocket]
    end
    subgraph Data
        P[(PostgreSQL)]
        R[(Redis Cache)]
    end
    subgraph External
        STRIPE[Stripe]
        S3[S3 Storage]
    end

    UI --> CDN
    CDN --> LB
    LB --> API
    LB --> WS
    API --> P
    API --> R
    WS --> R
    API --> STRIPE
    API --> S3
```

## Timeline

```mermaid
timeline
    title AI Coding Timeline
    2022 : ChatGPT launched
          : GitHub Copilot GA
    2023 : GPT-4 released
          : Claude 2
    2024 : Claude 3.5 Sonnet
          : GPT-4o
    2025 : Claude 4 Opus
          : AI coding agents
    2026 : Multi-agent swarms
          : Agentic workflows
```

## User Journey

```mermaid
journey
    title User onboarding flow
    section Sign up
        Open app     : 5: User
        Fill form    : 3: User
        Verify email : 1: System
    section Explore
        Dashboard    : 4: User
        First query  : 3: User
    section Engage
        Invite team  : 4: User
        Set up CI    : 2: User, System
```

## Quadrant / Mind Map

```mermaid
mindmap
  root((Project))
    Frontend
      React
      Tailwind
      TanStack Query
    Backend
      Rust
      Axum
      SQLite
    DevOps
      CI/CD
      Docker
      K8s
    Quality
      Tests
      Audit
      Docs
```

## Block Diagram

```mermaid
block-beta
    columns 3
    Input["User Input"]:1
    Process["Processor"]:1
    Output["Result"]:1
    DB[("Database")]:2
    Cache["Cache Layer"]:1

    Input --> Process
    Process --> Output
    Process <--> DB
    Process <--> Cache
```

## XY Chart

```mermaid
xychart-beta
    title "Monthly Revenue 2026"
    x-axis ["Jan", "Feb", "Mar", "Apr", "May", "Jun"]
    y-axis "Revenue (USD)" 0 --> 50000
    bar [12000, 18000, 22000, 28000, 35000, 42000]
    line [11000, 17000, 23000, 29000, 34000, 43000]
```

## Sankey (energy flow)

```mermaid
---
config:
  sankey:
    width: 600
    height: 300
---
sankey-beta
    Solar, Residential, 40
    Solar, Commercial, 30
    Wind, Residential, 25
    Wind, Commercial, 45
    Grid, Residential, 60
    Grid, Commercial, 50
    Residential, Used, 100
    Residential, Stored, 25
    Commercial, Used, 110
    Commercial, Stored, 15
```
