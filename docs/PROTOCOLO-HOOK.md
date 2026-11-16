# Extensão experimental da bandeja — protocolo v1

**Não há uma DLL de hook incluída.** Este documento define uma conexão opcional para um componente externo auditado e instalado explicitamente pelo usuário. Não modifica arquivos do Windows.

## Localização e ativação

`react-dock-tray-bridge.exe` precisa estar ao lado do `react_dock.exe` instalado. No modo estável, nunca é executado.

Ao selecionar *Experimental*, a React Dock executa `react-dock-tray-bridge.exe --snapshot` com stdin fechado e stdout capturado. A resposta precisa ser JSON v1. Se não houver resposta correta em 1,6 s, o modo não será ativado.

Exemplo:

```json
{"version": 1, "icons": [{"id":"aplicativo-exemplo","name":"Aplicativo","icon":null,"command":null,"tooltip":"Status"}]}
```

A React Dock limita os dados a 80 ícones, 1 MiB de stdout e tamanhos de campos; **comandos de execução vindos do helper são ignorados**. Os ícones fornecidos pela extensão são apenas mostrados (não repassamos cliques ou menus). Somente `data:image/png;base64,...` é aceito como ícone remoto do helper, até 240 KB.

## Recuperação

- 1 falha: mostra aviso; continua.
- 2 falhas: mostra aviso; continua.
- 3 falhas consecutivas: salva modo estável automaticamente.
- Sessão anterior não encerrada corretamente: inicia no modo estável.
- Usuário pode escolher **Usar estável** a qualquer momento.

A execução fora de processo reduz o alcance de falhas do helper **mas não neutraliza uma DLL já injetada no Explorer**. Uma implementação real deve incluir desinstalação do hook, watchdog externo, verificação de assinaturas, compatibilidade de versão e testes de recuperação antes de ser distribuída.
