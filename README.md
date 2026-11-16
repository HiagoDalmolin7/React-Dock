# React Dock 4.1.0 — correção de visibilidade e nitidez

Esta atualização prioriza a estabilidade ao trocar de aplicativo no Windows 11.

## O que mudou

- **Janela persistente:** sempre no topo, com checagem de visibilidade a cada dois segundos, sem roubar foco do programa em uso. Clicar nos aplicativos não deve ocultar a dock.
- **Instância única:** o plugin oficial de instância única impede duplicações e restaura a janela existente ao abrir o EXE novamente.
- **Fechamento seguro:** pedidos normais de fechamento são cancelados; para sair, use o menu de contexto do ícone do React Dock na bandeja do Windows.
- **Transparência externa corrigida:** removido o efeito Acrylic em toda a janela e a opção de redirecionamento que podiam criar um retângulo opaco ou problemas de composição. Os elementos não desenhados da WebView permanecem transparentes.
- **Ícones com mais definição:** tentativa de extrair 256 px, convertendo em PNG 160 px; ícones genéricos passam a ser SVGs vetoriais. Ícones de apps que só contenham versões 32/48 px ainda podem parecer suaves.
- **Ferramentas modernas:** Rust estável, Tauri 2 estável, Node 24, GitHub Actions v7 e runner Windows 2025. Não migra para Tauri 3 alfa.

> **Limites:** a transparência ao redor da dock foi priorizada. O efeito CSS de vidro é semitransparente, mas não corresponde a um Acrylic nativo recortado, com desfoque real de outras janelas. Esse recurso requer uma composição nativa separada e testes no Windows. O modo topo não se sobrepõe à tela segura do Windows ou a alguns jogos em tela cheia exclusiva. Não foi possível testar essa versão em um computador Windows neste ambiente.

## Atualizar com GitHub Desktop

1. Faça *Fetch origin* / *Pull origin* no GitHub Desktop antes de substituir os arquivos.
2. Extraia o ZIP da versão 4.1.0. Copie **o conteúdo extraído** para a pasta do repositório, substituindo arquivos; preserve `.git`.
3. Clique em *Commit to main* e *Push origin*, ou use a pull request disponibilizada no GitHub.
4. Acesse **Actions** e baixe o instalador/executável em *Artifacts* depois de uma compilação bem-sucedida.
5. **Encerre todas as instâncias da versão antiga** (Gerenciador de Tarefas), depois instale/execute o novo programa para evitar que o processo 4.0 continue aberto.

## Recuperar ou sair

- Abrir o executável de novo só deve focar a instância existente.
- Clique no ícone do React Dock na bandeja do Windows para restaurar a dock.
- Clique com o botão direito no ícone da bandeja e escolha **Sair do React Dock** para encerrar voluntariamente.

## Validação

`npm test` verifica estrutura, janela, transparência, ícones e workflow; `npm run build` gera o frontend; `cargo check` e `npm run tauri -- build --bundles nsis` são executados pelo GitHub Actions.

---

# React Dock — Liquid Glass para Windows 11

> **Descrição do repositório:** Dock flutuante para Windows 11, desenvolvida em Rust e Tauri, com visual Liquid Glass, bandeja personalizável, controles rápidos e foco em desempenho.

**Nome sugerido do repositório:** `React-Dock`

**Tópicos sugeridos:** `rust`, `tauri`, `windows-11`, `dock`, `liquid-glass`, `desktop-customization`

## Histórico: versão 4.0.1 — correção de compilação

React Dock é uma **dock flutuante e personalizável para Windows 11** inspirada no visual Liquid Glass. Projeto independente, inspirado em ideias de interface do Seelen UI, sem código-fonte copiado do Seelen.

> **Histórico da versão 4.0.1:** correção do erro Rust E0308 encontrado no primeiro GitHub Actions. Essa versão posteriormente compilou no GitHub. Não foi executada em Windows neste ambiente. A reprodução completa da bandeja de terceiros e os contadores reais de notificações **ainda não estão concluídos**.

## O que está implementado no código

- Rust + Tauri 2 + WebView2 + JavaScript/CSS: aplicação nativa para Windows com camada visual em WebView.
- Janela inferior centralizada, sempre à frente, sem usar `SHAppBarMessage(ABM_NEW)`: **não reserva área de trabalho**. Reposicionamento pelos limites físicos do monitor.
- **Na antiga 4.0.1**, a janela transparente usava Acrylic do Windows; **na 4.1.0 o efeito nativo foi retirado para corrigir a composição**. As camadas de vidro e o zoom animado permanecem.
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

## Primeiro upload pelo site do GitHub: atenção à pasta `.github`

Alguns modos de upload pelo navegador não incluem arquivos e pastas ocultas cujo nome começa com ponto. Se a pasta `.github` não aparecer na raiz do repositório, a compilação automática **não será iniciada**.

Para corrigir sem reenviar o projeto:

1. No repositório, clique **Add file > Create new file**.
2. No campo de nome, digite o caminho completo `.github/workflows/build-windows.yml`.
3. Cole o conteúdo do arquivo `build-windows.yml` deste projeto.
4. Clique **Commit changes**, escolhendo a branch `main`.
5. Abra **Actions**: a compilação é acionada pelo novo commit. Você também pode entrar no workflow e escolher **Run workflow**.

A compilação real depende do runner Windows no GitHub; mesmo com o workflow reconhecido podem surgir erros no código, que deverão ser corrigidos a partir dos logs.

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
