# 000 - Snapshot Inicial da Arquitetura

## Status
Aprovado

## Contexto
Este ADR documenta o estado atual da arquitetura do projeto `person-service-rust` conforme identificado através de engenharia reversa. O projeto foi iniciado com uma abordagem de "vibe coding" e agora busca formalizar suas estruturas para garantir consistência e manutenibilidade futuras.

## Decisões

### 1. Estrutura de Pastas
A estrutura de pastas segue uma divisão clara, alinhada com princípios de Arquitetura Hexagonal:
- `src/domain`: Contém as entidades de negócio e as interfaces (ports) de saída. Idealmente, deve ser totalmente agnóstica a infraestrutura.
- `src/application`: Contém os casos de uso (use cases) que orquestram a lógica de negócio, utilizando os ports definidos no domínio.
- `src/infrastructure`: Contém os adaptadores (adapters) de entrada (e.g., HTTP) e saída (e.g., PostgreSQL repository), que implementam os ports do domínio.
- `specs/api`: Destinado a especificações de API (e.g., OpenAPI).
- `specs/stories`: Destinado a user stories e critérios de aceite (e.g., Gherkin).
- `docs/adr`: Destinado a Architecture Decision Records.

### 2. Framework Web
- **Escolha:** `axum` com `tokio` como runtime assíncrono.
- **Justificativa:** `axum` é um framework web moderno e performático para Rust, construído sobre `tokio` e `tower`, oferecendo flexibilidade e um ecossistema robusto para aplicações assíncronas.

### 3. Banco de Dados (Escrita)
- **Escolha:** `PostgreSQL` acessado via `sqlx`.
- **Justificativa:** `PostgreSQL` é um banco de dados relacional maduro e amplamente utilizado, adequado para dados transacionais. `sqlx` é uma crate popular em Rust que oferece um ORM assíncrono robusto e seguro em tempo de compilação, com suporte a `PostgreSQL`.

### 4. Estratégia de Erros
- **Padrão:** Uso extensivo de `Result<T, E>` para tratamento de erros.
- **Crates:** `PersonError` (enum customizado) para erros de domínio/aplicação, `anyhow` para tratamento de erros em pontos de entrada da aplicação (ex: `main.rs`).
- **Justificativa:** A abordagem de `Result` é idiomática em Rust e permite um tratamento de erros explícito e seguro. `PersonError` centraliza os erros específicos do domínio, e `anyhow` simplifica o tratamento de erros em camadas superiores.

### 5. Multi-tenancy
- **Abordagem:** O `tenant_id` é explicitamente passado em operações de repositório para garantir isolamento de dados entre diferentes inquilinos.
- **Justificativa:** Essencial para aplicações SaaS, garantindo que os dados de um cliente não sejam acessados indevidamente por outro.

### 6. Geração de IDs
- **Abordagem:** `Uuid::new_v4()` é gerado no domínio (entidade `Person`).
- **Justificativa:** Mantém a posse da geração de identificadores únicos dentro da camada de domínio, garantindo que a entidade seja autocontida e independente de detalhes de infraestrutura para sua identificação.

## Dívida Técnica (Inicial)

Esta seção descreve pontos identificados na engenharia reversa que representam potenciais melhorias ou desvios do ideal da Arquitetura Hexagonal, mas que não serão abordados neste momento:

- **Dependências de Infraestrutura no Domínio:** A presença de `chrono` e `uuid` diretamente na entidade `Person` em `src/domain/entities/person.rs` introduz dependências de infraestrutura no módulo `domain`. O ideal seria injetar `Utc::now()` e `Uuid::new_v4()` através de interfaces (e.g., um `TimestampProvider` e um `IdGenerator`) nos `use cases`, mantendo o domínio puro.
- **Exposição de `AppState` no Adapter de Entrada:** O `person_handler.rs` na camada de `infrastructure/adapters/input/http` utiliza `State<AppState>`. Isso permite que o handler acesse dependências de infraestrutura que deveriam ser abstraídas. O ideal seria que o handler recebesse apenas o `UseCase` via um trait/interface, ou que o `AppState` fosse uma composição de traits de `UseCases`.
- **Lógica de Autenticação/Autorização no Handler:** A simulação da extração de `tenant_id` e `user_id` diretamente no `create_person` do `person_handler.rs` sugere que a lógica de autenticação e autorização pode estar acoplada ao handler. Essa responsabilidade deveria ser movida para um middleware, que enriqueceria a requisição com o contexto do usuário antes de chegar ao handler.
- **Mapeamento Genérico de Erros HTTP:** O tratamento de erros no `person_handler.rs` mapeia todos os `PersonError` para `500 Internal Server Error`. Para uma API RESTful mais robusta, seria ideal mapear tipos específicos de `PersonError` para `HTTP Status Codes` mais apropriados (e.g., `PersonError::Conflict` para `409 Conflict`, `PersonError::NotFound` para `404 Not Found`).