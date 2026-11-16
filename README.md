# React Dock 4.2.0 — menus estáveis, prévias e clique para minimizar

Esta versão reorganiza a interface para resolver os problemas observados no Windows 11: o painel que não abria, a dock que desaparecia ao tocar nos controles, o retângulo de fundo, a demora para identificar aplicativos e a ausência de prévias/minimização.

## Principais alterações

- **Dock permanente e menu separado:** a barra principal permanece em uma janela transparente de 116 px de altura. Bandeja, configurações e controles rápidos usam uma **segunda janela** transparente, acima da barra. Não se amplia nem se substitui a janela principal ao abrir um menu.
- **Configurações funcionais:** botões e abas passam a navegar pela janela de painel; botão fechar oculta apenas o painel, nunca a dock.
- **Fundo mais limpo:** removidos os gradientes decorativos e o `backdrop-filter` da superfície principal que criavam uma camada retangular na composição do WebView2. O fundo externo continua transparente e a superfície arredondada usa cor uniforme com opacidade ajustável.
- **Aplicativos mais responsivos:** a lista de janelas é consultada a cada **350 ms**, sem extrair/converter ícones a cada ciclo; imagens de aplicativos são carregadas sob demanda e ficam em cache.
- **Clique para minimizar:** clicar num aplicativo que estava ativo o minimiza; clicar de novo o restaura e traz para frente. Com múltiplas janelas do mesmo executável, a versão atual ainda escolhe uma janela representativa.
- **Prévia ao passar o mouse:** após um breve atraso, o Windows Desktop Window Manager tenta exibir uma miniatura viva numa pequena janela nativa, sem capturar a tela com permissões extras. Janelas protegidas, jogos exclusivos ou minimizados podem não fornecer conteúdo.
- **Segurança preservada:** nenhuma DLL é injetada no Explorer; o modo hook experimental continua desligado. Instância única e a opção de encerrar pela bandeja foram mantidas.
- **Ferramentas:** mantidas as versões estáveis utilizadas na 4.1.0 (Rust estável, Tauri 2, Node 24, GitHub Actions v7), evitando migrações para versões alfa.

> **Limitação importante:** esta revisão prioriza transparência sem retângulo e estabilidade. O **desfoque real do que está atrás** da superfície de vidro continua desativado; o acabamento é translúcido, mas não a refração óptica do Liquid Glass. Miniaturas DWM e o comportamento gráfico precisam de validação num Windows 11 real. Os testes automatizados de projeto não substituem o teste do aplicativo compilado.

## Como atualizar usando GitHub Desktop

1. No GitHub Desktop, abra o repositório `HiagoDalmolin7/React-Dock` e use **Fetch origin** / **Pull origin** se estiver disponível.
2. Extraia o ZIP **Somente alterações 4.2.0** e copie **o conteúdo das pastas** para a pasta local do repositório (`Repository → Show in Explorer`), permitindo substituir arquivos de nomes iguais. **Não toque na pasta `.git`**.
3. Confira as alterações em GitHub Desktop, escreva `React Dock v4.2.0 - Menus, transparencia, previews e minimizacao` no campo **Summary**.
4. Clique **Commit to main** e **Push origin**. Acompanhe a compilação em [GitHub Actions](https://github.com/HiagoDalmolin7/React-Dock/actions).
5. Encerre a versão antiga pelo ícone da bandeja (**Sair do React Dock**) antes de instalar a nova. Se houver erro de compilação, envie o log completo da Action.

## Checklist de teste no Windows 11

1. Abra **Configurações**, **Bandeja** e **Controles rápidos**: o painel deve surgir **acima** da dock, sem a dock sumir.
2. Feche o painel pelo `×` e abra de novo, alternando as abas.
3. Abra o Explorador ou navegador: os ícones devem aparecer na dock em menos de aproximadamente um ciclo de atualização, sujeito ao tempo do próprio Windows.
4. Com um programa na frente, clique em seu ícone para **minimizar**; clique outra vez para **restaurar**.
5. Passe o mouse sobre o ícone de programa aberto: a prévia DWM deve tentar aparecer acima da dock.
6. Teste a transparência com papel de parede claro e escuro; o retângulo externo não deve aparecer. O vidro interno deve ficar com uma cor uniforme.
7. Abra as configurações de aparência, modifique cor/tamanho/opacidade e confirme que a dock permanece visível.

## Validação local e limitações

`node --check src/main.js` e `node --test tests/project.test.mjs` foram executados. A compilação nativa Rust/Tauri, geração do instalador e teste visual do DWM dependem do GitHub Actions e de um PC com Windows 11. A prévia falha de modo seguro se o Windows não permitir renderização.

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
