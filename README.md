# React Dock v4.3.0 — Windows 11

Dock flutuante para Windows, construída com **Rust + Tauri 2 + WebView2**. Repositório de testes do projeto React Dock. Esta versão é experimental: compilações e testes de comportamento devem ocorrer em uma sessão Windows real.

## Novidades 4.3.0

- **Encerrar completamente:** em **Configurações → Encerrar React Dock**, mesmo quando o menu do ícone da bandeja nativa não abrir. A bandeja também exibe um menu com clique esquerdo ou direito, e duplo clique restaura a dock.
- **Uma dock por monitor conectado na inicialização**, mantendo a dock principal e criando `dock_1`, `dock_2`, etc. Em Configurações → Monitores, ajuste a visibilidade, a largura (automática ou de 5–100% da largura do monitor, respeitando o limite mínimo) e a distância da borda **independentemente**. Novos monitores conectados durante a execução exigem reinicialização nesta versão.
- **Sem sobreposição:** o bloco de ícones tem rolagem horizontal quando necessário. Os botões de bandeja/controles/configurações ficam ancorados à direita da dock.
- **Filtro de janelas auxiliares**, incluindo o host de entrada de texto do Windows, janelas ocultadas por DWM e janelas pertencentes a outras janelas sem `WS_EX_APPWINDOW`.
- **Várias instâncias do mesmo programa:** exibimos um único ícone com contador. Ao clicar, escolha individualmente qual janela abrir/minimizar.
- **Ícones com identidade consistente**, carregados por nome normalizado do executável e caminho real quando disponível; Explorer e Edge podem ser resolvidos mesmo sem estarem em execução. Alguns aplicativos empacotados/atalhos ainda podem mostrar ícone genérico.
- **Fixar e desafixar pela própria dock:** clique com o botão direito num aplicativo, abra *Gerenciar aplicativo* e selecione *Fixar na dock*.
- **Animações leves de ícone** na abertura/minimização e efeito de ampliação com componente vertical de acordo com a posição do mouse. **Não** é uma animação da janela nativa do outro aplicativo voando para a dock.
- **Iniciar personalizado**, acessível pelo ícone do Windows na dock ou pelo atalho **Ctrl + Alt + Espaço** global; procura entre fixados, programas abertos e atalhos `.lnk`/`.url` encontrados nos menus Iniciar do usuário e do computador. A tecla **Windows isolada continua abrindo o Iniciar nativo**, para preservar atalhos do sistema. A substituição completa exige integração com o shell, que não está ativada.
- **Controles rápidos com subpainéis** para Wi-Fi, Bluetooth e Volume. Ajuste de volume pelos botões; Wi-Fi/Bluetooth encaminham aos controles nativos, sem falsificar estado de rádio.
- **Fundo transparente da WebView**, com tinta translúcida somente no retângulo arredondado da dock, sem `backdrop-filter` ou gradiente externo. Não há desfoque nativo do desktop no momento; **a ausência de artefatos no WebView2 deve ser verificada no Windows**.

## Como atualizar pelo GitHub Desktop

1. Abra o GitHub Desktop com o repositório **HiagoDalmolin7/React-Dock** selecionado.
2. **Repository → Show in Explorer**.
3. Extraia **React-Dock-v4.3.0-Somente-Alteracoes.zip** e copie **o conteúdo** para a pasta do repositório, substituindo os arquivos do mesmo nome. Não copie o ZIP em si nem apague `.git`.
4. Volte ao GitHub Desktop. **Summary:** `React Dock v4.3.0 - Monitores, Iniciar e correções`.
5. **Commit to main → Push origin**. Acompanhe a compilação em https://github.com/HiagoDalmolin7/React-Dock/actions.
6. Encerre a instância antiga pela bandeja do Windows; se o menu falhar, finalize `react_dock.exe` no Gerenciador de Tarefas antes de executar a nova versão.

## Testes e restrições

- `npm install`, `npm test`, `npm run build`, `cargo check --manifest-path src-tauri/Cargo.toml`, `cargo test --manifest-path src-tauri/Cargo.toml`, `npm run tauri -- build --bundles nsis`.
- Os testes JavaScript de modelo e estrutura **não** equivalem a um teste de interface real do Windows. Compilação Windows não executada localmente neste ambiente; use o workflow.
- Não há injeção de DLL, substituição da barra de tarefas oficial, manipulação automática da tecla Windows isolada, leitura de contadores reais de notificações ou integração de bandeja de terceiros no modo estável.
- Hot-plug de monitores requer reiniciar a dock; aplicativos UWP/Store, janelas de jogos protegidas e janelas especiais podem não fornecer ícones e prévias DWM.
- O Windows sempre reserva determinados atalhos globais. Se Ctrl+Alt+Espaço estiver ocupado, continue usando o ícone Iniciar da dock.

Licença MIT.
