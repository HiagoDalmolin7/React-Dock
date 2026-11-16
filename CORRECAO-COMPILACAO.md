# React Dock 4.3.0 — correção do cargo check

**Motivo:** `unknown character escape` seis vezes, nas linhas 342 e 345 de `src-tauri/src/native.rs`.

**Correção:** os diretórios `%APPDATA%` e `%PROGRAMDATA%` do menu Iniciar passam a usar `r"Microsoft\Windows\Start Menu\Programs"` (string bruta Rust).

**Teste:** `tests/project.test.mjs` contém nova verificação para ambas as ocorrências.

## Atualização no GitHub Desktop

1. Extraia `React-Dock-v4.3.0-Correcao-Rust-Somente-Alteracoes.zip`.
2. Copie seu conteúdo para a pasta local do repositório, preservando as pastas. Substitua os arquivos existentes.
3. Summary: `React Dock 4.3.0 - Corrigir compilacao Rust do menu Iniciar`.
4. Clique em **Commit to main** e depois **Push origin**.
5. Acesse https://github.com/HiagoDalmolin7/React-Dock/actions.

Esta correção não altera o comportamento visual nem assegura a ausência de outras falhas de compilação; o próximo workflow verificará isso.
