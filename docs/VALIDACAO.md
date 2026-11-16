# Validação React Dock 4.3.0

## Realizado neste ambiente

- [x] Verificação de sintaxe do frontend com `node --check`.
- [x] 24 testes JavaScript/estruturais, incluindo testes de modelo puro para agrupamento por executável, larguras e wave vertical.
- [x] Validação de JSON para configuração, package e permissões do Tauri.
- [x] Inspeção dos pacotes ZIP e preservação das pastas ocultas `.github`.

## Ainda não comprovado

- [ ] Compilação Rust em Windows x64 com Tauri 2, incluindo criação de WebViews por monitor e funções Win32.
- [ ] Execução das docks em dois ou mais monitores com fatores DPI diferentes.
- [ ] Ausência de retângulo externo no WebView2 da máquina do usuário.
- [ ] Abre e fecha menu de bandeja em clique esquerdo e direito.
- [ ] Encerramento via Configurações e não reabertura por watchdog.
- [ ] Abrir e minimizar várias janelas do mesmo executável pelo seletor.
- [ ] Lista de atalhos do Iniciar e atalho global Ctrl+Alt+Espaço.
- [ ] Fixar/desafixar e consistência dos ícones de Explorer/Edge.
- [ ] Configurações rápidas e wave na direção vertical.

## Limitações intencionais

Tecla Windows **isolada** não substituída: o atalho global implementado é Ctrl+Alt+Espaço. A animação de minimização atual é somente sobre o ícone, não sobre a janela nativa do aplicativo. Sem desfoque real do desktop. Monitores adicionados/removidos exigem reiniciar o programa.

Se a Action falhar, encaminhar o log completo da primeira falha `error[E...]` ou `error:` e o trecho do arquivo que ela aponta.
