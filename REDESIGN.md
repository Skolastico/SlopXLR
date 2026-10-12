# GoXLR — interface integrada

A nova interface está em `daemon/web-content/redesign.html` e é incluída no executável do serviço pelo mecanismo de recursos existente. A rota principal `/` abre a nova interface. A interface anterior mantém-se em `index.html`, acessível por **Interface anterior (backup)**. O wrapper nativo existente abre a rota principal sem alterações.

## O que está ligado ao serviço

- Dispositivo e perfil reais; carregar e guardar perfil.
- Volumes dos oito canais e saídas, mute dos faders e atribuições dos canais.
- Microfone: tipo, ganho, bandas EQ e frequências com limites próprios por modelo, gate completo, compressor completo e De-esser.
- Memórias EQ A/B com a mesma chave de armazenamento local da integração anterior.
- Encaminhamento através da matriz real do serviço.
- Iluminação: cores e apresentação dos faders, botões e estados desligados, acento/global, animação e, no GoXLR completo, sampler, encoders e ecrãs.
- Submix A/B, volumes ligados, misturas por saída e seleção de monitor.
- Efeitos: seis presets, biblioteca, Reverb, Echo, Pitch, Gender, Megafone, Robot e HardTune, com restrições por estilo.
- Sampler: bancos A/B/C, comportamento e ordem por botão, biblioteca, reprodução/paragem, cortes e pré-gravação.
- Controlos físicos: função do mute temporário, Bleep e scribbles do GoXLR completo.
- Criar, carregar, guardar como e eliminar perfis do dispositivo/microfone; carregar apenas cores; abrir pastas para importar/exportar biblioteca; ganho dos áudios.
- Preferências do serviço/dispositivo, ações ao encerrar/suspender/retomar e firmware pela Internet ou ficheiro local, com estado de progresso e confirmação antes da atualização.
- Ações e aplicações com páginas, posições, gravação e resultado do teste de ações.
- Estado do modo ativo e diagnóstico.

As alterações de volume/processamento/encaminhamento são imediatas. Ações e aplicações têm rascunhos e gravação explícita. Testar uma ação executa-a realmente. As configurações não são migradas para um formato novo: o frontend utiliza os contratos existentes.

## Limites e compatibilidade

- O serviço atual suporta ações personalizadas e aplicações por páginas apenas no Mini. A interface não disponibiliza essas gravações para o GoXLR completo.
- Estas opções já não dependem de um iframe da interface antiga. A ligação **Interface anterior** continua disponível como alternativa.
- A organização dos ficheiros usa as pastas existentes do serviço. A atualização por ficheiro usa o caminho local que o serviço valida.
- A gravação de samples mantém os gestos dos botões físicos; a interface mostra quando o dispositivo está a gravar.
- As funções exclusivas do GoXLR completo são apresentadas apenas quando existem no estado recebido.
- Anterior/Seguinte percorrem a página em edição. A página ativa no hardware é lida do serviço e muda pelos gestos físicos existentes; não foi inventado um comando de ativação.
- Alterações em ações/aplicações noutro frontend são verificadas antes de guardar para evitar substituir uma configuração divergente.
- As memórias A/B aplicam vários comandos consecutivos. Uma falha pode deixar uma aplicação parcial; a interface apresenta o erro e volta a consultar o dispositivo.
- Não foram exercitados comandos USB nem ações Windows durante os testes desta entrega. A confirmação física do Mini e do GoXLR completo continua necessária.

## Experimentar sem reiniciar o serviço

Com o GoXLR Utility personalizado aberto:

```powershell
python redesign-dev.py
```

Abre `http://localhost:8767/redesign.html`. Este anfitrião local serve os novos ficheiros e encaminha os pedidos ao serviço já aberto na porta 14564. Só escuta em loopback, rejeita hosts/origens externos e não envia comandos no arranque. Encaminha comandos de dispositivo, uma lista explícita de preferências e os pedidos normais de firmware; recusa encerrar o serviço ou forçar a validação de firmware. Ctrl+C encerra-o. A interface anterior é aberta diretamente na origem do serviço, conservando o websocket existente.

## Versão ativa e recuperação

A compilação está ativa em `../GoXLR-Personalizado/goxlr-daemon.exe`. O serviço usa o ficheiro `config/settings.json` desse pacote. A app nativa abre diretamente a nova interface; o atalho de arranque do Windows foi atualizado para esse executável e configuração.

O backup completo anterior à ativação encontra-se em `../GoXLR-Personalizado/backup-interface-anterior-20261011-200050`, incluindo executáveis, configurações, perfis, biblioteca e o atalho de arranque anterior. O atalho `Restaurar-interface-anterior.cmd` encerra normalmente o serviço e repõe apenas o executável anterior, conservando os perfis/configurações atuais e o executável novo. A interface anterior também pode ser usada pelo link dentro da app, sem trocar o serviço.

## Validação

```powershell
node --test --test-isolation=none tests/redesign-model.test.mjs tests/redesign-render.test.mjs tests/redesign-migration.test.mjs
```

O teste Rust `goxlr-ipc --test redesign_contract` descodifica os comandos das opções migradas com os tipos reais do serviço, incluindo as variantes sem argumentos. Os testes cobrem serialização IPC, erros HTTP/erros devolvidos em HTTP 200, escala de volume, posições ocupadas, validação de ações, texto não confiável, dados reais de renderização e diferenças Mini/GoXLR. A compilação release utiliza `.tools/build-goxlr.cmd build --release -p goxlr-daemon` a partir da pasta de trabalho principal.

## Primeira prioridade — 11 outubro 2026

Cor global corrigida para SetGlobalColour; leitura real do microfone e histórico com limiar do gate; EQ simples/detalhado, reposição e eixo logarítmico baseado nas frequências; listas visuais ordenáveis para ações de encerramento/suspensão/retoma, conservando comandos personalizados e rascunhos.

21 testes Node passaram. Compilação release instalada e recursos servidos confirmados. Leitura de microfone confirmada no Mini ligado; não foram exercitados a reposição do EQ nem mudanças de iluminação durante a verificação. O GoXLR completo continua a exigir validação física. Backup desta atualização em ../GoXLR-Personalizado/backup-prioridade-1-20261011-214522; backup original preservado.

## Resposta dos botões físicos — 11 outubro 2026

Mute normal dos faders aplicado ao soltar, sem esperar os 350 ms de reconhecimento do duplo clique. Canais sem associações não abrem temporizador de navegação; nos canais associados, um segundo clique repõe o mute anterior antes de entrar em Aplicações. O temporizador apenas expira, sem repetir o mute. Limpeza das janelas pendentes ao entrar/sair do modo preserva os outros canais.

13 testes Rust release passaram, incluindo USB simulado para mute imediato, expiração e entrada em aplicações a partir de canal com/sem mute. A verificação física de latência deve ser feita premindo o botão no dispositivo.

## Sampler e soundboard do Mini — 12 outubro 2026

Biblioteca com navegação por pastas, pré-escuta, forma de onda e corte por botão sem alterar o original. Bancos e posições mantêm ordem visual estável. Caminhos Windows canónicos e nomes com caracteres de glob são resolvidos como ficheiros literais, dentro da biblioteca.

O Mini passa a expor três bancos virtuais de quatro botões, com vários áudios e ordem sequencial/aleatória. A reprodução usa o motor de áudio do serviço e a saída System do GoXLR Mini, conservando o encaminhamento escolhido pelo utilizador. As ações Reproduzir/Parar soundboard podem ser atribuídas às páginas A–D/Bleep em Controlos; funcionam através do serviço, independentemente da janela. Não são apresentados gravação física nem efeitos inexistentes no Mini. O Full mantém o sampler existente. A pré-escuta utiliza a saída predefinida do navegador/WebView.

23 testes Node e 16 testes Rust passaram. Validação no Mini ligado: ficheiro WAV temporário numa subpasta com nome literal [1], descodificação da onda, reprodução com estado ativo e paragem sem erro, corte 0.1–99.9% confirmado na interface/serviço. Ficheiro e associação de teste retirados; banco A reposto. Volumes e mutes iguais ao estado anterior, perfis AXAXAOLA/DEFAULT conservados. A utilização dos novos comandos pelos botões físicos e o GoXLR Full requerem confirmação física.

Compilação release instalada, recursos novos servidos e janela nativa reaberta. Backup anterior em ../GoXLR-Personalizado/backup-soundboard-mini-20261012-003811. A recuperação para o binário antigo guarda as novas ações num ficheiro de configurações separado e converte apenas as ações incompatíveis para Sem ação, para permitir arrancar o serviço antigo.

## Paridade da interface — 12 outubro 2026

Quatro diferenças resolvidas: vistas de EQ/gate/compressor/frequências usam as preferências reais do perfil do microfone; aplicação de cores e estilos dos faders a todos e cores/estado desligado aos grupos de botões; decay de Reverb com passos representáveis (10 ms até 1000 ms, 100 ms acima); encerramento em Definições com diálogo interno, cancelamento, bloqueio de rascunhos e estado de serviço encerrado. Guardar perfil do microfone conserva as vistas após reiniciar o serviço.

26 testes Node e um teste Rust do contrato IPC passaram. Verificados no Mini: mudança de vista e conservação após recarregar; comando real de estado desligado do grupo; cancelamento do encerramento, encerramento real, estado final e reinício do serviço. Configurações de apresentação originais repostas. Não foi exercitado o Reverb num GoXLR Full físico. Backup anterior em ../GoXLR-Personalizado/backup-paridade-20261012-011402.

## Estabilidade no arranque do Mini — 12 outubro 2026

A primeira leitura dos faders USB inicializa a referência física sem substituir os volumes restaurados. Só o movimento posterior altera o volume. Regressão coberta no teste de mute/navegação: volume 130 conservado para posição inicial 37 e repetição 37, depois alterado para 42 ao mover. 26 testes frontend e 16 Rust passaram. Reinício real confirmou a conservação de volumes, perfis, mutes, processamento, encaminhamento, iluminação, sampler, ações e aplicações. Atalho do Windows verificado; reinício completo do Windows e pressão física dos botões continuam pendentes. Ver ESTABILIDADE-MINI.md.
