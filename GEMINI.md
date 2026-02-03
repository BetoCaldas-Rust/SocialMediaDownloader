# Social Media Downloader - Diretrizes do Projeto (GEMINI.md)

Este arquivo define as regras de conduta e convenções técnicas que o Antigravity (IA) deve seguir neste projeto para garantir assertividade e evitar suposições.

## Regras de Conduta (Anti-Suposições)

1. **PROVA ANTES DA AÇÃO**: Antes de propor ou implementar uma solução para um erro, use ferramentas (`ls`, `run_command`, `view_file`) para verificar a existência de arquivos, tamanhos de binários e estruturas de diretórios. Nunca suponha que um arquivo "deve estar lá" ou que um path padrão de ferramenta (ex: /resources) existe sem checar.
2. **SIMPLICIDADE POR PADRÃO**: Não adicione camadas extras de complexidade (sistemas de log customizados, bibliotecas de diagnóstico, scripts de auxílio) a menos que o problema atual não possa ser resolvido de forma mais direta.
3. **EVIDÊNCIA DE SUCESSO**: Sempre que afirmar que um problema foi "corrigido", forneça evidências técnicas e mensuráveis (ex: "O instalador antes tinha 4MB e agora tem 26MB", ou "O comando ls confirmou que o binário está na pasta X").
4. **BLOQUEIO POR DÚVIDA**: Se o caminho de um arquivo ou um comportamento do sistema for desconhecido, pare e peça ao usuário para rodar um comando de inspeção (ex: `dir /s`, `ls -R`) ou envie uma foto, em vez de tentar "adivinhar" o comportamento.
5. **FONTE DA VERDADE**: O arquivo `start.bat` na raiz do projeto é a referência de como o aplicativo deve se comportar. O instalador final e o código Rust devem espelhar exatamente a lógica de execução desse script.

## Estrutura Técnica de Referência

- **Frontend**: Rust (egui/eframe). Deve rodar como aplicação de console para visibilidade de logs durante o debug do instalador.
- **Backend**: Python (PyInstaller --onefile). Deve ser lançado pelo Rust via `Command` ou `cmd /C start`.
- **Instalador**: Gerado via `cargo packager`. Deve obrigatoriamente incluir o binário do backend (`smd-backend.exe`) como recurso.
