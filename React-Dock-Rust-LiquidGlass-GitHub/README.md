# React Dock — Liquid Glass para Windows 11

> **Descrição do repositório:** Dock flutuante para Windows 11, desenvolvida em Rust e Tauri, com visual Liquid Glass, bandeja personalizável, controles rápidos e foco em desempenho.

**Nome sugerido do repositório:** `React-Dock`

**Tópicos sugeridos:** `rust`, `tauri`, `windows-11`, `dock`, `liquid-glass`, `desktop-customization`

## Versão 4.0 (prévia de desenvolvimento)

React Dock é uma **dock flutuante e personalizável para Windows 11** inspirada no visual Liquid Glass. Projeto independente, inspirado em ideias de interface do Seelen UI, sem código-fonte copiado do Seelen.

> **Status:** versão de código-fonte para primeiro build no GitHub Actions. Não foi compilada/testada em Windows neste ambiente. A reprodução completa da bandeja de terceiros e os contadores reais de notificações **ainda não estão concluídos**.

## O que está implementado no código

- Rust + Tauri 2 + WebView2 + JavaScript/CSS: aplicação nativa para Windows com camada visual em WebView.
- Janela inferior centralizada, sempre à frente, sem usar `SHAppBarMessage(ABM_NEW)`: **não reserva área de trabalho**. Reposicionamento pelos limites físicos do monitor.
- Janela transparente e efeito **Acrylic** do Windows, com camadas de vidro, reflexo e zoom animado nos ícones (melhor esforço conforme versão do Windows e GPU).
- Lista de janelas Win32, ativação, atalhos fixados, ícones extraídos dos executáveis quando possível, fixar/remover e mover os ícones.
- Bandeja **própria editável**: atalhos adicionados ou removidos pelo usuário e ícone da React Dock registrado na bandeja nativa como recuperação.
- Controles rápidos: volume ↑/↓/mudo direto; atalhos para Wi-Fi, Bluetooth, modo avião, áudio, tela, notificações, não perturbe e energia. O painel Win+A também é acessível.
- Bateria e relógio. Personalização de cor, tamanho, transparência, distância da borda e animações.
- **Supervisão do modo experimental**, desligado por padrão. Só funciona com helper verificado e instalado separadamente; falha 3 vezes → modo estável; queda/interrupção da sessão → estável na próxima inicialização.

## Não confundir com funcionalidade concluída

- **Hook real dentro de `explorer.exe` NÃO é distribuído**. O painel oferece a opção e a infraestrutura de isolamento/retorno, mas permanece indisponível sem um auxiliar externo compatível, testado e confiável. Não copie DLLs de fontes desconhecidas.
- A bandeja própria contém atalhos configuráveis; **não espelha todos os ícones nativos de terceiros**. Um helper experimental pode fornecer snapshots para visualização (somente leitura nesta versão).
- Números de mensagens não lidas por aplicativo e piscadas provenientes de notificações externas não estão integrados. Campos de atenção legados são lidos, mas não geramos contagens artificiais. Isso precisa de API de notificação, permissões e integração individual conforme aplicativo.
- Wi-Fi/Bluetooth acessam os painéis oficiais; não há interruptor próprio que altere os rádios diretamente. Acesso privilegiado e restrições do sistema variam.
- O Windows pode restringir primeiro plano, ícones de aplicações protegidas e tarefas executadas como administrador. Esta build não tem suporte completo a múltiplos monitores.

## Publicar e compilar no GitHub (recomendado)

1. Crie um repositório GitHub para React Dock (ou use seu repositório próprio).
2. Extraia `React-Dock-Rust-LiquidGlass-GitHub.zip` e envie **todos os arquivos para a raiz**, incluindo `.github/workflows/build-windows.yml`, `src/`, `src-tauri/` e `package.json`.
3. Acesse **Actions → React Dock Rust - Build Windows → Run workflow**.
4. O fluxo instala Rust stable, Node 22, dependências npm e compila o instalador NSIS e o executável; baixe os artefatos ao final.
5. No seu Windows, marque **Configurações → Personalização → Barra de tarefas → Comportamentos → Ocultar automaticamente a barra de tarefas** para que as janelas maximizadas possam ficar atrás da dock.
6. Primeiro teste com o **modo estável**. Se houver erros de compilação, envie o log completo da Action.

O instalador gerado por este projeto não está assinado digitalmente. O SmartScreen pode avisar. Nunca desative o antivírus para instalar.

### Compilar localmente

Instale Rust MSVC, Visual Studio Build Tools, Microsoft Edge WebView2 Runtime e Node.js 22. Em seguida, rode `scripts\testar.bat` para desenvolvimento ou `scripts\compilar.bat` para build.

## Proteções do modo experimental

O supervisor (em `src-tauri/src/bridge.rs`) só procura `react-dock-tray-bridge.exe` **ao lado do executável da React Dock**; não baixa nem injeta bibliotecas. Ao selecionar *Ativar experimental*, verifica o helper com um snapshot antes de salvar. Em execução, o frontend solicita snapshots a cada 7 segundos, com timeout de 1,6 s. Três falhas consecutivas desativam o modo experimental; na próxima abertura após encerramento anormal, também é desativado.

Protocolo do helper: veja [`docs/PROTOCOLO-HOOK.md`](docs/PROTOCOLO-HOOK.md). Ele não está incluído, e nunca há garantia de que um hook no Explorer seja livre de falhas.

## Arquitetura

```text
src/main.js + src/style.css   interface e efeitos do Liquid Glass
src-tauri/src/native.rs       janelas, ícones, ShellExecute, volume e bateria
src-tauri/src/settings.rs     preferências e persistência
src-tauri/src/bridge.rs       opt-in, timeout e fallback do hook experimental
src-tauri/src/lib.rs          comandos da interface e posicionamento
.github/workflows            compilação Windows e artefatos
```

## Referências

- [Seelen UI](https://github.com/eythaann/Seelen-UI): referência de arquitetura, dock e bandeja.
- [Tauri v2 — configuração de janelas](https://v2.tauri.app/reference/config/): Acrylic/Windows e janela transparente.
- [Microsoft — Shell_NotifyIcon](https://learn.microsoft.com/windows/win32/api/shellapi/nf-shellapi-shell_notifyiconw): APIs documentadas para publicar ícones, não para enumerar todos os ícones de terceiros.

Licença: MIT para os arquivos originais React Dock desta distribuição.

## Identidade do projeto

**React Dock** é o nome do aplicativo; a interface atual utiliza JavaScript/CSS com Tauri, não o framework React. O código-fonte mantém a arquitetura e as limitações documentadas da prévia v4.

Ao instalar esta versão, as configurações novas ficam em `%APPDATA%\ReactDock\config.json`. Na primeira execução, caso ainda não existam, o programa tenta importar preferências de `%APPDATA%\NovaDock\config.json` sem apagar os dados antigos.
