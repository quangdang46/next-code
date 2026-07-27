# Mermaid Complete Reference

## 1. Flowchart — Luồng cơ bản

```mermaid
flowchart TD
    A[Start] --> B{Is it working?}
    B -->|Yes| C[Great!]
    B -->|No| D[Debug]
    D --> E[Fix bug]
    E --> B
    C --> F[Ship it]
    F --> G((Done))
```

## 2. Sequence Diagram — Tương tác giữa các thành phần

```mermaid
sequenceDiagram
    participant User
    participant App
    participant API
    participant DB

    User->>App: Click "Save"
    App->>API: POST /api/save
    API->>DB: INSERT record
    DB-->>API: OK
    API-->>App: 200 Success
    App-->>User: ✅ Saved
```

## 3. Class Diagram — OOP

```mermaid
classDiagram
    class Animal {
        +String name
        +int age
        +makeSound() void
    }
    class Dog {
        +fetch() void
    }
    class Cat {
        +purr() void
    }
    Animal <|-- Dog
    Animal <|-- Cat
    class Owner {
        +String name
        +adopt(Animal a) void
    }
    Owner --> Animal : owns
```

## 4. Git Graph

```mermaid
gitGraph
    commit id: "init"
    branch feature
    checkout feature
    commit id: "add auth"
    commit id: "add UI"
    checkout main
    merge feature
    commit id: "release v1"
```

## 5. Gantt Chart — Timeline dự án

```mermaid
gantt
    title Project Timeline
    dateFormat  YYYY-MM-DD
    section Design
    Wireframes      :done, 2026-07-01, 3d
    Mockups         :done, 2026-07-04, 2d
    section Dev
    Frontend        :active, 2026-07-06, 5d
    Backend         :2026-07-08, 5d
    section QA
    Testing         :2026-07-13, 3d
```

## 6. Entity Relationship — Database

```mermaid
erDiagram
    USER ||--o{ ORDER : places
    ORDER ||--|{ LINE_ITEM : contains
    PRODUCT ||--o{ LINE_ITEM : includes
    USER {
        int id PK
        string name
        string email
    }
    ORDER {
        int id PK
        int user_id FK
        date created_at
    }
    LINE_ITEM {
        int id PK
        int order_id FK
        int product_id FK
        int quantity
    }
    PRODUCT {
        int id PK
        string name
        float price
    }
```

## 7. State Diagram — Trạng thái

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Loading : fetch data
    Loading --> Success : response OK
    Loading --> Error : timeout
    Success --> Idle : refresh
    Error --> Loading : retry
    Error --> Idle : dismiss
    Success --> [*] : unmount
```

## 8. Pie Chart — Biểu đồ tròn

```mermaid
pie title Stack Overflow Survey 2026
    "JavaScript" : 32
    "TypeScript" : 28
    "Python" : 24
    "Rust" : 10
    "Go" : 6
```

## 9. Timeline

```mermaid
timeline
    title AI Coding Evolution
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

## 10. User Journey

```mermaid
journey
    title User onboarding
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

## 11. Mindmap

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

## 12. Block Diagram

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

## 13. XY Chart

```mermaid
xychart-beta
    title "Monthly Revenue 2026"
    x-axis ["Jan", "Feb", "Mar", "Apr", "May", "Jun"]
    y-axis "Revenue (USD)" 0 --> 50000
    bar [12000, 18000, 22000, 28000, 35000, 42000]
    line [11000, 17000, 23000, 29000, 34000, 43000]
```

## 14 năng lượng

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

## 15. Package Diagram — C

```mermaid
packages-beta
    package "Frontend" {
        package "UI Components" {
            component Button
            component Table
            component Modal
        }
        package "Hooks" {
            component useAuth
            component useQuery
        }
    }
    package "Backend" {
        package "API" {
            component auth_handler
            component user_handler
        }
        package "DB" {
            component models
            component migrations
        }
    }
```

## 16. Requirement Diagram — Yêu cầu

```mermaid
requirementDiagram
    requirement req_auth {
        id: REQ-001
        text: "User must be able to login"
        risk: high
        verifymethod: test
    }
    requirement req_2fa {
        id: REQ-002
        text: "Support 2FA"
        risk: medium
        verifymethod: test
    }
    element login_form {
        type: interface
    }
    element otp_handler {
        type: backend
    }
    req_auth <- element login_form
    req_2fa <- element otp_handler
    req_2fa -refines -> req_auth
```

## 17. C4 Context — Kiến trúc tổng thể

```mermaid
C4Context
    title System Context
    Person(user, "User", "End user")
    System(sys, "App", "Main system")
    System_Ext(email, "Email Service", "SendGrid")
    System_Ext(payment, "Payment", "Stripe")

    Rel(user, sys, "Uses")
    Rel(sys, email, "Sends email")
    Rel(sys, payment, "Charges")
```

## 18. C4 Container — Chi tiết containers

```mermaid
C4Container
    title Container diagram
    Person(user, "User")
    Container_Boundary(app, "App") {
        Container(web, "Web App", "Next.js", "UI")
        Container(api, "API", "Rust/Axum", "Backend")
        ContainerDb(db, "DB", "PostgreSQL")
    }
    Rel(user, web, "Uses", "HTTPS")
    Rel(web, api, "Fetches", "REST")
    Rel(api, db, "Reads/Writes", "SQL")
```

## 19. Quadrant Chart

```mermaid
quadrantChart
    title Technology Radar
    x-axis "Low Value" --> "High Value"
    y-axis "Complex" --> "Easy"
    quadrant-1 "Adopt"
    quadrant-2 "Evaluate"
    quadrant-3 "Hold"
    quadrant-4 "Retire"
    React: [0.8, 0.7]
    Rust: [0.9, 0.4]
    JQuery: [0.2, 0.8]
    Webpack: [0.3, 0.2]
```

## 20. Flowchart Subgraphs — Luồng có nhóm

```mermaid
flowchart LR
    subgraph Auth
        A[Login] --> B[Token]
    end
    subgraph Data
        C[Fetch] --> D[Parse]
    end
    subgraph Render
        E[Component] --> F[DOM]
    end
    B --> C
    D --> E
```

## 21. Flowchart với styling

```mermaid
flowchart LR
    A[Start] --> B[Process]
    B --> C{Check}
    C -->|Pass| D[✅ Done]
    C -->|Fail| E[❌ Error]

    style A fill:#4ade80,stroke:#16a34a,color:#000
    style C fill:#fbbf24,stroke:#d97706,color:#000
    style D fill:#4ade80,stroke:#16a34a,color:#000
    style E fill:#f87171,stroke:#dc2626,color:#000
```

## 22. Sequence với notes và activation

```mermaid
sequenceDiagram
    participant A as Client
    participant B as Server
    participant C as DB

    A->>+B: POST /data
    Note over A,B: SSL/TLS
    B->>+C: INSERT
    C-->>-B: row_id
    Note right of B: cache result
    B-->>-A: 201 Created
```

## 23. State Diagram với fork/join

```mermaid
stateDiagram-v2
    state fork_state <<fork>>
    state join_state <<join>>

    [*] --> fork_state
    fork_state --> Task1
    fork_state --> Task2
    Task1 --> join_state
    Task2 --> join_state
    join_state --> Done
    Done --> [*]
```
