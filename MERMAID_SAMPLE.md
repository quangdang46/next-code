# Mermaid Sample Diagrams

This file contains various mermaid diagram samples for testing the mermaid sidebar renderer.

## Flowchart

```mermaid
graph TD
    A[Start] --> B{Is it working?}
    B -->|Yes| C[Great!]
    B -->|No| D[Debug]
    D --> B
    C --> E[Deploy]
```

## Sequence Diagram

```mermaid
sequenceDiagram
    participant User
    participant Agent
    participant LLM
    User->>Agent: Ask question
    Agent->>LLM: Send prompt
    LLM-->>Agent: Return response
    Agent-->>User: Display answer
```

## Class Diagram

```mermaid
classDiagram
    class Animal {
        +String name
        +int age
        +makeSound() void
    }
    class Dog {
        +String breed
        +fetch() void
    }
    class Cat {
        +String color
        +purr() void
    }
    Animal <|-- Dog
    Animal <|-- Cat
```

## State Diagram

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Processing: User input received
    Processing --> Thinking: LLM call
    Thinking --> Processing: Partial response
    Processing --> Idle: Response complete
    Idle --> [*]
```

## Gantt Chart

```mermaid
gantt
    title Project Timeline
    dateFormat  YYYY-MM-DD
    section Planning
    Requirements    :a1, 2026-07-01, 3d
    Design          :a2, after a1, 5d
    section Development
    Frontend        :b1, after a2, 7d
    Backend         :b2, after a2, 7d
    section Testing
    Integration     :c1, after b1, 3d
    UAT             :c2, after c1, 2d
```

## Pie Chart

```mermaid
pie title Languages Used
    "Rust" : 45
    "TypeScript" : 30
    "Python" : 15
    "Go" : 10
```

## Git Graph

```mermaid
gitGraph
    commit
    branch feature
    checkout feature
    commit
    commit
    checkout main
    commit
    merge feature
    commit
```

## Entity Relationship Diagram

```mermaid
erDiagram
    USER ||--o{ POST : writes
    POST ||--o{ COMMENT : has
    USER ||--o{ COMMENT : writes
    USER {
        int id PK
        string name
        string email
    }
    POST {
        int id PK
        string title
        string body
        int user_id FK
    }
    COMMENT {
        int id PK
        string text
        int post_id FK
        int user_id FK
    }
```

## Timeline

```mermaid
timeline
    title Project Milestones
    2026 Q1 : MVP Launch
             : Core features
    2026 Q2 : Beta Testing
             : User feedback
    2026 Q3 : Public Release
             : Scale infrastructure
```

## Mindmap

```mermaid
mindmap
  root((Next Code))
    Agent Runtime
      Tools
      Providers
      Compaction
    UI
      TUI
      Grok Face
      Sidebar
    Storage
      SQLite
      Notepad
      Memory
```

## Block Diagram

```mermaid
block-beta
    columns 3
    Input["User Input"]:1
    Processor["Agent Core"]:1
    Output["Response"]:1
    space:1
    LLM["LLM Provider"]:1
    space:1
    Input --> Processor
    Processor --> Output
    Processor --> LLM
    LLM --> Processor
```

## Complex Flowchart

```mermaid
graph LR
    subgraph Frontend
        A[React UI] --> B[API Gateway]
    end
    subgraph Backend
        B --> C[Auth Service]
        B --> D[Agent Service]
        B --> E[Storage Service]
    end
    subgraph External
        D --> F[OpenAI]
        D --> G[Anthropic]
        E --> H[(PostgreSQL)]
    end
    style A fill:#e1f5fe
    style B fill:#fff3e0
    style C fill:#f3e5f5
    style D fill:#e8f5e9
    style E fill:#fce4ec
    style F fill:#ffccbc
    style G fill:#d7ccc8
    style H fill:#b3e5fc
```
