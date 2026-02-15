# User Story: Cadastro de Pessoa

Como um usuário do sistema,
Eu quero cadastrar uma nova pessoa,
Para que suas informações fiquem armazenadas e acessíveis.

## Critérios de Aceite

### Cenário: Cadastro de pessoa individual com sucesso
Dado que eu forneço os dados de uma pessoa individual (nome, CPF, dados de pessoa individual)
E que não existe outra pessoa com o mesmo CPF para o mesmo `tenant_id`
Quando eu solicito o cadastro da pessoa
Então a pessoa deve ser criada com um ID único e os dados fornecidos
E a resposta deve ser `201 Created` com os dados da pessoa criada.

### Cenário: Cadastro de pessoa jurídica com sucesso
Dado que eu forneço os dados de uma pessoa jurídica (nome, CNPJ, nome fantasia, dados de pessoa jurídica)
E que não existe outra pessoa com o mesmo CNPJ para o mesmo `tenant_id`
Quando eu solicito o cadastro da pessoa
Então a pessoa deve ser criada com um ID único e os dados fornecidos
E a resposta deve ser `201 Created` com os dados da pessoa criada.

### Cenário: Cadastro de pessoa estrangeira com sucesso
Dado que eu forneço os dados de uma pessoa estrangeira (nome, número de passaporte, código do país, dados de pessoa estrangeira)
E que não existe outra pessoa com o mesmo número de passaporte para o mesmo `tenant_id`
Quando eu solicito o cadastro da pessoa
Então a pessoa deve ser criada com um ID único e os dados fornecidos
E a resposta deve ser `201 Created` com os dados da pessoa criada.

### Cenário: Tentativa de cadastro de pessoa com documento já existente
Dado que já existe uma pessoa com um determinado documento (CPF, CNPJ ou passaporte) para o `tenant_id`
Quando eu tento cadastrar uma nova pessoa com o mesmo documento e `tenant_id`
Então a pessoa não deve ser criada
E a resposta deve ser `500 Internal Server Error` (PersonError::Conflict) indicando o conflito.

### Cenário: Erro interno no servidor durante o cadastro
Dado que ocorre um erro inesperado durante o processo de persistência (e.g., erro no banco de dados)
Quando eu solicito o cadastro de uma pessoa
Então a pessoa não deve ser criada
E a resposta deve ser `500 Internal Server Error` (PersonError::DatabaseError) indicando o problema.

## Dívida Técnica

- **Geração de Uuid e `created_at` no Domínio:** A geração de `Uuid::new_v4()` e `chrono::Utc::now()` diretamente na entidade `Person` em `src/domain/entities/person.rs`, embora mantenha a posse do ID no domínio, introduz dependências de infraestrutura (`uuid` e `chrono`) no módulo `domain`. O ideal seria que essas informações fossem injetadas através de um construtor ou um `TimestampProvider` no `UseCase`, mantendo o domínio puro.
- **Acesso direto a `AppState` no Handler:** O `person_handler.rs` na camada de `infrastructure/adapters/input/http` está utilizando `State<AppState>`, o que significa que o handler tem acesso a todo o estado da aplicação, que pode conter dependências de infraestrutura. Isso viola o princípio de injeção de dependência e expõe o handler a detalhes que não deveriam ser de sua responsabilidade. O ideal seria que o handler recebesse apenas o `UseCase` ou uma interface mais específica que oculte a complexidade do `AppState`.
- **Simulação de `tenant_id` e `user_id`:** A simulação da extração de `tenant_id` e `user_id` no `create_person` do `person_handler.rs` deve ser substituída por uma lógica real de autenticação e autorização via middleware. A responsabilidade de obter essas informações deve ser de um componente anterior ao handler, que passaria esses dados de forma limpa para o caso de uso, mantendo o handler focado em sua função principal.
- **Tratamento de Erros Genérico no Handler:** O `match` no `person_handler.rs` retorna um `StatusCode::INTERNAL_SERVER_ERROR` genérico para qualquer `PersonError`. Idealmente, diferentes tipos de `PersonError` deveriam ser mapeados para `HTTP Status Codes` mais específicos (ex: `PersonError::Conflict` para `409 Conflict`, `PersonError::NotFound` para `404 Not Found`).