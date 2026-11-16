# Validação da prévia React Dock v4

Realizadas neste ambiente:
- Verificação de sintaxe JavaScript com `node --check`.
- `npm test` (5 testes estáticos de arquitetura, modo estável, isolamento, fallback).
- Verificação de JSON, integridade de pacote ZIP e existência dos arquivos.

Não realizadas aqui:
- Compilação Rust (`cargo check`) — Rust toolchain não instalado.
- `npm run build` — dependências npm indisponíveis e acesso à registry bloqueado no container.
- Teste do instalador, do WebView2, do Acrylic, dos controles do sistema e do Explorer num Windows 11.

A compilação completa e a compatibilidade real deverão ser verificadas pelo workflow Windows do GitHub Actions incluído. **Não classificar esta versão como pronta para produção.**
