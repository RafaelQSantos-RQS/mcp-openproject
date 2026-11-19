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

## 🛠️ Catálogo de Ferramentas (18 Tools)

| Ferramenta | Descrição | Principais Parâmetros |
| :--- | :--- | :--- |
| `list_projects` | Lista projetos acessíveis com suporte a paginação | `page`, `page_size` |
| `get_project` | Detalhes completos de um projeto | `id_or_identifier` |
| `search_work_packages` | Pesquisa e filtra work packages (tarefas, bugs, épicos) | `query`, `project_id`, `type_id`, `status`, `assignee_id`, `sprint_id`, `version_id`, `page`, `page_size` |
| `get_work_package` | Recupera detalhes completos de um work package por ID (incluindo progresso, estimativas, sprint, versão e tempo gasto) | `id` |
| `create_work_package` | Cria um novo work package (suporta subtarefas via `parent_id`, alocação em `sprint_id`, versão em `version_id`, estimativa, trabalho restante e % concluído) | `project_id`, `type_id`, `subject`, `parent_id`, `sprint_id`, `version_id`, `estimated_time`, `remaining_time`, `percentage_done`, `description`, `status_id`, `priority_id`, `assignee_id`, `custom_fields` |
| `update_work_package` | Atualiza atributos de um work package existente (alocação em `sprint_id`, `version_id` (0 para desassociar), progresso, estimativa, trabalho restante, etc.) | `id`, `parent_id`, `sprint_id`, `version_id`, `lock_version` (opcional), `estimated_time`, `remaining_time`, `percentage_done`, `subject`, `description`, `status_id`, `type_id`, etc. |
| `list_types` | Lista os tipos de work packages suportados (Tarefa, Bug, etc.) | *(Nenhum)* |
| `list_statuses` | Lista os status disponíveis no fluxo de trabalho | *(Nenhum)* |
| `get_current_user` | Obtém dados do usuário autenticado pela API key (ID, nome, email, login, admin) | *(Nenhum)* |
| `list_project_assignees` | Lista usuários elegíveis para receber atribuição de tarefas no projeto | `project_id` |
| `add_project_member` | Associa um usuário a um projeto como membro com papel/role | `project_id`, `user_id`, `role_id` (opcional, padrão: 9 para 'Member') |
| `list_time_entry_activities` | Lista as atividades disponíveis para lançamento de horas (ex: Development, Specification) | `project_id` (opcional), `work_package_id` (opcional) |
| `log_time` | Lança horas gastas em uma tarefa (apontamento de tempo com recálculo automático de `spentTime`) | `work_package_id`, `hours` (ex: `'2h'`, `'1.5h'`, `'30m'`, `'PT2H'`), `comment`, `spent_on`, `activity_id`, `user_id` |
| `list_sprints` | Lista sprints cadastrados, com suporte a filtro por projeto | `project_id` (opcional) |
| `get_sprint` | Obtém detalhes de um sprint por ID (nome, status, datas de início/fim e projeto definidor) | `id` |
| `create_version` | Cria uma nova versão / milestone em um projeto OpenProject | `project_id`, `name`, `description`, `start_date`, `end_date`, `status` (`'open'`, `'locked'`, `'closed'`), `sharing` (`'none'`, `'descendants'`, etc.) |
| `list_versions` | Lista versões / milestones cadastrados, com suporte a filtro por projeto ou global | `project_id` (opcional) |
| `get_version` | Obtém detalhes de uma versão / milestone por ID (nome, status, datas, compartilhamento e projeto definidor) | `id` |

> 💡 **Formatos de Duração Amigáveis:**
> O servidor converte automaticamente formatos comuns de duração para o padrão ISO 8601 do OpenProject:
> - `'4h'` ou `'4'` ➔ `PT4H` (4 horas)
> - `'2.5h'` ou `'2,5h'` ➔ `PT2H30M` (2 horas e 30 minutos)
> - `'1h30m'` ou `'01:30'` ➔ `PT1H30M`
> - `'45m'` ou `'45min'` ➔ `PT45M` (45 minutos)
> - Formatos ISO diretos (`'PT2H'`, `'PT30M'`) também são aceitos.

---

## 📥 Instalação Rápida (Binários Pré-compilados)

Baixe, descompacte e instale a versão mais recente diretamente em `~/.local/bin/` com um único comando:

### Linux (x86_64)
```bash
mkdir -p ~/.local/bin && \
  curl -sL https://github.com/RafaelQSantos-RQS/mcp-openproject/releases/latest/download/mcp-openproject-linux-x86_64.tar.gz | tar -xz -C ~/.local/bin && \
  chmod +x ~/.local/bin/mcp-openproject
```

### macOS (Apple Silicon - ARM64)
```bash
mkdir -p ~/.local/bin && \
  curl -sL https://github.com/RafaelQSantos-RQS/mcp-openproject/releases/latest/download/mcp-openproject-macos-arm64.tar.gz | tar -xz -C ~/.local/bin && \
  chmod +x ~/.local/bin/mcp-openproject
```

> 💡 Certifique-se de que `~/.local/bin` esteja no seu `$PATH` (padrão moderno na maioria dos ambientes Linux e macOS).

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
# Usando o binário instalado em ~/.local/bin
~/.local/bin/mcp-openproject --test-connection

# Ou a partir do código-fonte compilado
cargo run -- --test-connection
```

Se as credenciais estiverem corretas, você verá uma mensagem de confirmação em `stderr`:
```text
Successfully connected to OpenProject at https://seu-openproject.exemplo.com
```

---

## 🤖 Integração com Clientes MCP

### Claude Desktop / Cursor / Antigravity

Adicione à configuração de servidores MCP (ex: `claude_desktop_config.json` ou arquivo de settings do seu assistente):

```json
{
  "mcpServers": {
    "openproject": {
      "command": "mcp-openproject",
      "env": {
        "OPENPROJECT_URL": "https://seu-openproject.exemplo.com",
        "OPENPROJECT_API_KEY": "seu_token_aqui"
      }
    }
  }
}
```

> 💡 Se `mcp-openproject` não estiver no `PATH` do aplicativo, utilize o caminho absoluto:
> `"command": "/home/SEU_USUARIO/.local/bin/mcp-openproject"` (ex: `/home/rafael_qsantos/.local/bin/mcp-openproject`).

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
