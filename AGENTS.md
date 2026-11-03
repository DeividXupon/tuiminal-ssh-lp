# Tuiminal SSH LP

Landing page do Tuiminal acessível por SSH. O visitante roda `ssh <host>` e, em vez
de um shell, recebe uma TUI desenhada com Ratatui dentro da sessão SSH. É a versão
"terminal" da LP web; o produto é o mesmo.

## O que é o Tuiminal

Workspace de desenvolvimento dentro do terminal, código aberto, em beta
(`npm install --global tuiminal@beta`). Feito com Bun, OpenTUI, React e tuiparts.
Cinco ferramentas em abas, trocadas com `Alt+1–5`:

| Aba | Ferramenta | Cor (paleta Prime) |
| --- | --- | --- |
| Alt+1 | Database: catálogo, grid, inspector, SQL, transações (MySQL/MariaDB, PostgreSQL, SQLite, MySQL MCP) | `#a78bfa` |
| Alt+2 | Git: diffs locais, PRs, Issues e notificações do GitHub | `#f7c873` |
| Alt+3 | Runner: descobre comandos, roda serviços, acompanha logs | `#64d8ff` |
| Alt+4 | HTTP: monta, salva, envia e automatiza requests | `#fb923c` |
| Alt+5 | Term Agents: shells e CLIs em seções de até dois terminais | `#5ee6a8` |

Fontes de verdade, nesta ordem:

- `~/Projects/tuiminal`: o produto (`README.md` / `README.pt-BR.md`, `AGENTS.md`,
  `packages/core`, `packages/feature-*`, `apps/cli`). Comandos, atalhos e
  nomes das features vêm daqui.
- `~/Projects/tuiminal-lp`: a LP web em Astro, publicada em
  https://tuiminal-lp.deivid-silva-info.workers.dev/pt-br/ (Cloudflare Workers).
  Copy em pt-BR, estética ASCII, JetBrains Mono, paleta em `src/data/features.ts`.
  Tom: conversacional e factual, sem slogans nem superlativos. Esta LP SSH deve
  seguir o mesmo tom, cores e vocabulário.

## Este projeto

Rust (edition 2024). `russh` serve SSH, `ratatui` desenha, `tokio` roda tudo.
Um único arquivo: `src/main.rs`.

- `SshWriter`: `Write` que acumula bytes do Ratatui e envia pelo canal SSH via
  `Handle::data` numa task própria.
- `Client`: um `Handler` por visitante. `auth_none` aceita qualquer usuário sem
  senha. `channel_open_session` cria o `Terminal` com viewport fixo; `pty_request`
  e `window_change_request` redimensionam; `shell_request` chama `render`.
- `render`: hoje só escreve `tuiminal` centralizado. É o único lugar com UI.
- `data`: `q` ou `Ctrl+C` fecham o canal. Não há loop de eventos nem estado além
  do terminal; qualquer interatividade nova precisa de um modelo de input próprio.
- `main`: carrega `host.key` do diretório atual e escuta em `127.0.0.1:2222` com
  timeout de inatividade de 300 s.

Rodar localmente:

```sh
ssh-keygen -t ed25519 -N '' -f host.key   # uma vez; não versionar
cargo run
ssh -p 2222 localhost
```

## Regras

- Nunca versionar chaves: `host.key`, `vps-key`, `vps-key.pub` ficam fora do git.
- O servidor aceita qualquer login sem senha por design. Nunca executar comandos,
  abrir shell real, ler arquivos do host ou expor nada além da LP a partir do
  input do visitante.
- Conteúdo e copy seguem a LP web e o README do Tuiminal. Ao mudar o canal de
  release (beta, pré-alfa) ou atalhos, conferir lá primeiro.
- `cargo build` e `cargo clippy` antes de commit. Commit e push só com pedido
  explícito.
- Próximos passos prováveis: seções por ferramenta com as cores acima, navegação
  por teclado, comando de instalação, e deploy na VPS atrás de porta 22.
