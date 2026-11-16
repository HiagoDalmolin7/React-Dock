# Validação da React Dock 4.2.0

## Verificações executadas

- Sintaxe JavaScript: `node --check src/main.js` — aprovada.
- Testes locais: `node --test tests/project.test.mjs` — **15 de 15** verificações de estrutura e regressão aprovadas.
- JSON de Tauri, package.json e capability: leitura válida.
- Integridade do pacote ZIP: verificação antes da entrega.

## Não validado neste ambiente

- `cargo check` / `cargo test` para o código Win32/Tauri revisado (a ferramenta Rust/Windows não está instalada neste ambiente).
- Compilação de instalador e EXE no Windows, transparência da WebView2 e o resultado visual da miniatura DWM.
- Funcionamento em monitores múltiplos, modos HDR, janelas de terceiros protegidas e aplicativos minimizados.

## Pontos críticos da arquitetura

- A janela `dock` nunca cresce para 550 px; `panel` usa uma WebView separada, inicialmente invisível.
- A API `show_panel` atualiza o pedido de página; `get_panel_request` garante a navegação mesmo quando a página ainda está carregando.
- A enumeração de janelas ficou rápida porque não converte ícones no processo de atualização. Ícones são buscados separadamente e cacheados.
- `toggle_window` utiliza `IsIconic`, `ShowWindow(SW_MINIMIZE)` e `SetForegroundWindow`.
- O preview usa `DwmRegisterThumbnail` / `DwmUpdateThumbnailProperties` e é removido quando o mouse sai; janelas não renderizáveis são ignoradas.
- O modo experimental de bandeja permanece sem injeção no Explorer.
