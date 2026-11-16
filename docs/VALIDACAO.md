# Validação da React Dock 4.1.0

## Testado neste ambiente

- Sintaxe JavaScript: `node --check src/main.js` — sem erros.
- `node --test tests/*.test.mjs` — 10 de 10 testes estáticos aprovados.
- JSON da interface Tauri e do `package.json` — parse válido.
- Arquivos ZIP (completo e apenas alterações) — verificação de integridade.

## Ainda depende do GitHub Actions e de um Windows 11 real

- Compilar o Rust, o frontend atualizado para Vite 8 e o instalador NSIS.
- Confirmar a persistência da dock quando outros aplicativos recebem foco.
- Verificar transparência da janela e nitidez dos ícones em diferentes escalas DPI.
- Testar prevenção de instâncias duplicadas e opção de sair pela bandeja.

## Observações técnicas

- Os helpers sem uso `exe_for_hwnd` e `foreground_exe` foram removidos para eliminar os avisos `dead_code` do Rust.
- `actions/checkout`, `actions/setup-node` e `actions/upload-artifact` foram atualizados para a série v7 com Node 24.
- A versão 4.1.0 não utiliza efeito Acrylic nativo de janela inteira por segurança visual; o acabamento do vidro depende das camadas CSS.
- O programa não utiliza hooks no Explorer por padrão, e as limitações das notificações/bandeja permanecem.
