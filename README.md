# mcp-openproject

Servidor de **Model Context Protocol (MCP)** em **Rust** de alta performance e portabilidade para integração com o **OpenProject** (API v3).

Desenvolvido para permitir que assistentes de IA (Claude Desktop, Antigravity, Cursor, etc.) consultem, criem e gerenciem tarefas e projetos no OpenProject com baixo consumo de tokens de contexto e schemas ergonômicos.

---

## 🚀 Funcionalidades

- **Transporte Padrão `stdio`**: Total conformidade com o protocolo MCP sobre canais de entrada e saída padrão, utilizando o SDK oficial `rmcp` (v3.11).
- **Sanitização HAL+JSON**: Abstrai a verbosidade do formato hipermidiático da API v3 do OpenProject, entregando dados limpos e amigáveis para modelos de linguagem.
- **Concorrência Otimista Inteligente**: Atualizações automáticas de `lockVersion` sem quebrar o fluxo do LLM.
- **Portabilidade Total**: Compilado com `reqwest` + `rustls`, eliminando dependências externas do OpenSSL do sistema operacional.
- **Isolamento de I/O**: Logs e diagnósticos são canalizados exclusivamente para `stderr` (via `tracing`), blindando `stdout` para as mensagens JSON-RPC.
- **Modo de Diagnóstico**: Flag `--test-connection` para validar credenciais antes de subir o servidor em produção.

---

## 🛠️ Catálogo de Ferramentas (11 Tools)

| Ferramenta | Descrição | Principais Parâmetros |
| :--- | :--- | :--- |
| `list_projects` | Lista projetos acessíveis com suporte a paginação | `page`, `page_size` |
| `get_project` | Detalhes completos de um projeto | `id_or_identifier` |
| `search_work_packages` | Pesquisa e filtra work packages (tarefas, bugs, épicos) | `query`, `project_id`, `type_id`, `status`, `assignee_id`, `page`, `page_size` |
| `get_work_package` | Recupera detalhes completos de um work package por ID | `id` |
| `create_work_package` | Cria um novo work package em um projeto (suporta subtarefas via `parent_id`) | `project_id`, `type_id`, `subject`, `parent_id`, `description`, `status_id`, `priority_id`, `assignee_id`, `custom_fields` |
| `update_work_package` | Atualiza atributos de um work package existente | `id`, `parent_id`, `lock_version` (opcional), `subject`, `description`, `status_id`, `type_id`, etc. |
| `list_types` | Lista os tipos de work packages suportados (Tarefa, Bug, etc.) | *(Nenhum)* |
| `list_statuses` | Lista os status disponíveis no fluxo de trabalho | *(Nenhum)* |
| `get_current_user` | Obtém dados do usuário autenticado pela API key (ID, nome, email, login, admin) | *(Nenhum)* |
| `list_project_assignees` | Lista usuários elegíveis para receber atribuição de tarefas no projeto | `project_id` |
| `add_project_member` | Associa um usuário a um projeto como membro com papel/role | `project_id`, `user_id`, `role_id` (opcional, padrão: 9 para 'Member') |

---

## ⚙️ Configuração

O servidor pode ser configurado por variáveis de ambiente ou através de um arquivo `.env` na raiz:

```env
OPENPROJECT_URL="https://seu-openproject.exemplo.com"
OPENPROJECT_API_KEY="seu_token_api_aqui"
```

> **Como obter seu token de API no OpenProject:**
> 1. Acesse sua conta no OpenProject (`/my/account`).
> 2. No menu lateral, clique em **Tokens de acesso**.
> 3. Em **Chaves de API**, gere uma nova chave e copie o valor.

---

## 🧪 Verificação e Diagnóstico

Para testar a conectividade com sua instância do OpenProject antes de usá-lo com agentes de IA:

```bash
cargo run -- --test-connection
```

Ou usando o binário compilado:

```bash
./target/release/mcp-openproject --test-connection
```

Se as credenciais estiverem corretas, você verá uma mensagem de confirmação em `stderr`:
```text
Successfully connected to OpenProject at https://seu-openproject.exemplo.com
```

---

## 🤖 Integração com Clientes MCP

### Claude Desktop

Adicione ao seu `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "openproject": {
      "command": "/caminho/absoluto/para/mcp-openproject/target/release/mcp-openproject",
      "env": {
        "OPENPROJECT_URL": "https://seu-openproject.exemplo.com",
        "OPENPROJECT_API_KEY": "seu_token_aqui"
      }
    }
  }
}
```

---

## 🏗️ Compilação e Testes

```bash
# Executar suíte completa de testes (unitários + integração mockada)
cargo test

# Executar linter estrito
cargo clippy --all-targets -- -D warnings

# Build otimizado para produção
cargo build --release
```
