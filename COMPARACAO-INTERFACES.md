# Comparação da interface nova com a anterior

Verificação: 12 outubro 2026. Comparação do frontend anterior incluído em daemon/web-content/assets/index-BJM830Pw.js com os módulos em daemon/web-content/redesign, os comandos gerados em tests/redesign-commands.json e os respetivos percursos no serviço. Mini ligado: S210201775DI7, perfis AXAXAOLA / DEFAULT. Esta comparação não equivale a validação física de todos os controlos.

## Cobertura

| Área | Estado na nova interface |
| --- | --- |
| Mixer | Volumes, saídas, mutes, atribuições, destinos de mute e submix presentes. |
| Microfone | Tipo, ganho, EQ por modelo, frequências, gate, compressor, De-esser, memórias A/B, reposição do EQ e leitura do sinal presentes. Vistas simples/detalhadas persistidas pelo serviço. |
| Encaminhamento | Matriz do serviço e seleção de monitor presentes. No Mini, o soundboard utiliza System, não uma entrada física Samples. |
| Iluminação | Ajustes individuais, global/acento, animações e funções exclusivas do Full presentes. Aplicação de cores e estilos a grupos disponível. |
| Efeitos Full | Presets e parâmetros presentes; decay de Reverb utiliza apenas os valores representáveis. Não disponíveis fisicamente no Mini. |
| Sampler Full | Bancos, ordem, comportamentos, cortes, ganho, reprodução/paragem e pré-gravação presentes; gravação acionada no hardware. |
| Soundboard Mini | Novo recurso: bancos virtuais, vários áudios por botão, ordem sequencial/aleatória, corte, pré-escuta e ações de reproduzir/parar. |
| Perfis e biblioteca | Criar, carregar, guardar como, eliminar, carregar cores e abrir pastas para importar/exportar presentes. |
| Controlos / Aplicações | Páginas separadas, avançar/recuar, posições, cores e gravação presentes no Mini. O serviço restringe as duas extensões ao Mini, também na implementação anterior. |
| Definições | Preferências, ações de encerramento/suspensão/retoma, diagnóstico e atualização de firmware presentes. Encerramento com confirmação interna e recuperação de ficheiros de origem disponíveis. |

## Diferenças identificadas na comparação inicial (pontos 1–4 já resolvidos)

### 1. Conservar e clarificar as vistas do microfone

A antiga interface grava Simple/Advanced para NoiseGate, Compressor, Equaliser e EqFineTune através de SetElementDisplayMode. A nova usa eqSimple=true em memória, alterna esse valor localmente e usa secções details; não lê nem grava essas preferências de apresentação. Fechar/reabrir repõe a vista do EQ e a organização anterior de gate/compressor não tem correspondência completa.

Implementar seleção explícita simples/detalhada para cada grupo, inicializada pelo estado do dispositivo e persistida pelo contrato existente; manter todos os parâmetros acessíveis. Critério de aceitação: depois de fechar/reabrir, os grupos conservam a vista escolhida; mudar a vista não altera o áudio nem apaga ajustes.

Evidência: app.mjs (ui.eqSimple e data-eq-mode), features.mjs (micDetails), chamadas antigas SetElementDisplayMode.

### 2. Recuperar aplicação da iluminação a grupos

A antiga oferece SetAllFaderColours, SetAllFaderDisplayStyle, SetButtonGroupColours e SetButtonGroupOffStyle. A nova permite configurar os elementos individualmente, mas não oferece equivalentes para aplicar a mesma configuração ao conjunto. É uma perda de conveniência, não a ausência do controlo individual.

Implementar Aplicar a todos os faders e aplicar a grupos de botões, incluindo seletores/tipos de efeitos no Full. Critério: um comando aplica a configuração escolhida ao grupo e o estado recebido confirma os elementos alterados.

Evidência: features.mjs (lightingDetails), app.mjs (iluminação principal), chamadas antigas acima.

### 3. Afinar a precisão dos controlos dos efeitos Full

O decay do Reverb é apresentado por um range linear em milissegundos com passo 1. O serviço só representa múltiplos de 10 ms até 1000 ms e múltiplos de 100 ms acima disso; set_decay_millis quantiza o valor. A interface pode apresentar durante o ajuste um valor que o dispositivo não conserva. A antiga utiliza uma lista de valores/índices. A faixa global válida continua 10–20000 ms.

Recuperar a correspondência entre posição do controlo e valor representável; confirmar o valor final pelo serviço. Não classificar todos os parâmetros de efeitos como ausentes: os comandos estão gerados nos contratos da interface nova.

Evidência: features.mjs (effectSchemas.reverb), profile/src/components/reverb.rs (set_decay_millis), antigo ReverbEffect.decay_map.

### 4. Encerrar o serviço pela interface

A interface anterior oferece StopDaemon com confirmação. Na nova não há ação equivalente. A saída pelo ícone da aplicação continua disponível. Implementar uma ação clara Encerrar serviço, com confirmação de interrupção do áudio e estado de encerramento coerente.

### 5. Validação física e expansão de compatibilidade

O Full ainda exige testes físicos para efeitos, sampler, ecrãs e iluminação. As páginas personalizadas de Controlos/Aplicações continuam restritas ao Mini no serviço; suportar essas extensões no Full é desenvolvimento adicional, não uma regressão introduzida pelo redesign.

## Verificação do percurso dos botões do Mini

monitor_inputs lê os botões USB, encaminha-os para LayerTracker e executa a ação da página correspondente. PlaySample seleciona uma faixa respeitando a ordem do botão virtual e passa-a ao motor de áudio; StopSample usa a paragem do mesmo banco/botão. A execução pertence ao serviço e não depende do frontend aberto.

Pressão longa no Cough durante 700 ms entra/sai do modo de ações. Com esse modo ativo, dois cliques no Cough avançam e três recuam; a janela de resolução é 350 ms. A–D e Bleep executam as cinco ações da página. O modo Aplicações é distinto e impede a ativação simultânea do modo de ações. O mute normal dos faders continua aplicado ao soltar, sem esperar a janela de duplo clique.

Os 16 testes Rust passaram nesta revisão, incluindo reconhecimento dos gestos, posse dos botões através da mudança de modo, persistência de ações, contrato de Play/Stop e mute imediato com USB simulado. Os 23 testes de frontend também passaram. O Mini está ligado e expõe o sampler por software. A reprodução/paragem e o corte com um WAV temporário foram verificados na instalação anterior; nesta revisão não foram alterados os áudios, os perfis nem as ações atuais.

### Confirmação física pendente

1. Associar um áudio a um botão virtual e guardar o perfil.
2. Associar Reproduzir/Parar em Controlos e guardar as ações.
3. Manter Cough premido e confirmar Ações no cabeçalho; premir o botão atribuído e ouvir o som.
4. Com duas páginas configuradas, testar dois/três cliques no Cough.
5. Sair com pressão longa e verificar que um mute de fader silencia imediatamente.
6. Fechar apenas a janela e repetir a reprodução com o serviço ainda aberto.

Esta lista exige intervenção do utilizador no hardware; os testes automáticos não confirmam a experiência acústica nem substituem a pressão física dos botões.

## Implementação dos quatro pontos — 12 outubro 2026

As vistas de EQ, gate, compressor e frequências usam SetElementDisplayMode e leem settings.display. A escolha mantém-se ao reabrir a janela; Guardar perfil do microfone conserva-a no perfil após reiniciar o serviço. Os comandos de apresentação não alteram os parâmetros de áudio.

Aplicação de cores/estilos de qualquer fader a todos e aplicação de cores/estado desligado aos grupos FaderMute, EffectSelector e EffectTypes. A fonte é o último estado confirmado pelo serviço.

Reverb decay passa a uma lista de 10 em 10 ms até 1000 ms e 100 em 100 ms depois, até 20000 ms; o texto e o comando usam o valor real, não o índice do controlo.

Encerrar serviço em Definições, com confirmação interna acessível, proteção de rascunhos e indicação de serviço encerrado. O utilizador pode cancelar; ao confirmar executam-se as ações de encerramento existentes do serviço.

26 testes de frontend e um teste Rust do contrato IPC passaram. A validação física do Full permanece pendente.

## Recomparação e conclusão da migração — 12 outubro 2026

A comparação dos comandos usados pelo frontend anterior, dos controlos gerados pela interface nova e do contrato IPC identificou RecoverDefaults como a função ainda ausente. Foi adicionada em Definições → Recuperar ficheiros de origem, para Profiles, MicProfiles, Presets e Icons. O serviço invoca goxlr-defaults sem --overwrite: conserva os ficheiros existentes e adiciona os de origem em falta. Não carrega perfis nem altera o áudio. Uma pasta isolada recebeu cinco perfis de origem; a segunda extração conservou um ficheiro com conteúdo deliberadamente diferente.

Status/Patch são mensagens do transporte anterior, não controlos em falta. GetStatus corresponde à consulta HTTP /api/get-devices da interface nova. Esta continua a atualizar o estado periodicamente; não foi adicionada nesta fase uma migração para WebSocket.

Falhas de contacto com o serviço e dispositivo USB desligado têm instruções distintas. Falhas de leitura de ações e aplicações são acompanhadas separadamente: a área que funciona continua a atualizar, os rascunhos são conservados, os dados antigos são identificados e a gravação da área sem leitura confirmada fica bloqueada até recuperar. Erros de transporte e timeout oferecem instruções de recuperação; um timeout não afirma que a operação deixou de ser aplicada.

Validação: 31 testes de frontend e um teste de contrato IPC passaram. Recuperação de ficheiros verificada apenas em pasta isolada. Testes acústicos, gestos físicos, efeitos/sampler do Full e reinício completo do Windows continuam pendentes para a fase final, por pedido do utilizador. Compatibilidade das páginas de ações/aplicações com o Full continua a ser expansão adicional; não estava disponível no serviço anterior.

## Atualizações em tempo real — 12 outubro 2026

A nova interface subscreve /api/websocket e pede GetStatus para confirmar a ligação. Notificações Status/Patch desencadeiam os leitores HTTP existentes, agrupadas numa janela de 50 ms; eventos recebidos durante uma leitura ou gravação originam uma atualização posterior. Mantém-se a consulta periódica de 1,5 s como recuperação e para a descoberta das aplicações no Windows. A ligação WebSocket tenta recuperar automaticamente; é encerrada ao fechar a página ou encerrar o serviço.

Mudanças de runtime das ações/aplicações e resultados das ações são notificadas pelo serviço através de um watch channel. Só diferenças reais notificam: leituras idênticas não geram eventos. A mensagem usa um JSON Patch vazio, válido para a interface antiga, e a nova consulta os endpoints separados dos controlos. Não foi alterado o contrato de comandos nem o percurso de mute físico/áudio.

Volumes, estado dos mutes e volumes/pickup das aplicações são atualizados sem substituir os campos em edição. Um botão com foco deixa de bloquear a atualização visual destes valores. Os rascunhos continuam conservados pelo processo de leitura existente.

Verificação: 32 testes de frontend e 16 testes do serviço passaram. Numa interface isolada com comandos de hardware desativados, duas alterações de volume foram refletidas em 80 e 76 ms, contra 358 e 374 ms na versão anterior com consulta periódica. A segunda alteração manteve o botão de mute focado e atualizou o volume de 78% para 50%. Estes números medem a simulação local da interface, não a latência acústica nem garantem tempos idênticos em todas as máquinas. Testes de som e gestos físicos continuam reservados para a fase final.

## Volume das aplicações e canal físico do Mini — 12 outubro 2026

Tentativa inicial, substituída pela separação descrita na secção seguinte depois de o utilizador reportar bloqueio dos volumes.

O modo Aplicações consumia os movimentos dos faders sem alterar o perfil, mas não contrariava o volume que o firmware do Mini aplica diretamente ao canal atribuído. Assim, o fader C podia alterar Música inteiro mesmo estando a controlar apenas a aplicação do slot C.

No Mini, cada movimento neste modo envia primeiro SetChannelVolume com o nível atual do perfil para o canal original do fader, antes de encaminhar o valor para a sessão da aplicação. Isto também ocorre durante pickup e em slots vazios, em todas as páginas. Se a reposição USB falhar, a leitura não é marcada como consumida, permitindo repetir a reposição mesmo com o fader parado. Ao sair, volta o percurso normal de controlo do canal. Não se introduzem atribuições USB não documentadas nem canais fictícios.

Esta é uma reposição pelo serviço após a leitura física, não uma desativação da alteração autónoma do firmware. A existência de uma variação acústica transitória ainda precisa de verificação no teste final com som. O comando não é aplicado ao Full neste percurso porque SetChannelVolume também move os seus faders motorizados; a expansão do modo Aplicações para esse modelo continua pendente.

Verificação: 16 testes do serviço passaram. A simulação USB altera explicitamente os canais antes de entregar os movimentos, verificando reposição nos quatro faders, encaminhamento individual A/B/C, slot vazio D, pickup, segunda página, repetição após falha USB e regresso ao controlo normal. Nenhuma aplicação de teste foi executada e não se reproduziu áudio.

## Separação dos faders e LEDs das aplicações no Mini — 12 outubro 2026

Foi removida a reposição contínua de SetChannelVolume durante os movimentos. Esse comando interfere com o latch do Mini e a leitura devolvida pelo firmware pode deixar de acompanhar a posição física. O teste anterior entregava diretamente posições físicas ao daemon e não reproduzia este efeito.

Ao entrar no modo Aplicações, o serviço escolhe quatro canais de entrada sem associações a aplicações e sem áudio audível: volume principal e submix a zero, ou nenhum encaminhamento para saídas. Exclui o microfone. No perfil atual são Game, Console, LineIn e Sample. Desliga temporariamente todas as saídas e os feeds internos destes canais no hardware e usa-os como sensores dos quatro faders. As atribuições, volumes e encaminhamento guardados no perfil permanecem intactos. Mic, Chat, Music e System continuam com os seus níveis anteriores. Se não houver quatro canais elegíveis, recusa a entrada e apresenta uma explicação; não sacrifica canais em uso.

A aquisição inicial exige baixar os faders ao mínimo para assumir o latch dos canais temporários; depois cada fader deve cruzar o volume atual da aplicação. Após essa aquisição, movimentos consecutivos continuam a chegar à sessão Windows sem comandos de volume de hardware a bloquear as leituras. Páginas mantêm pickup por aplicação. Saída do modo, edição de hardware, alteração das associações, suspensão e encerramento repõem canais, volumes, encaminhamento e modo dos LEDs. Falhas durante a preparação tentam repor o estado; falhas nessa reposição pedem reinicialização da ligação com o perfil guardado.

As barras usam TwoColour, com a cor da aplicação e volume do canal isolado que acompanha o fader adquirido. Pickup ou ausência de sessão mantém a barra ténue; mute apaga-a e slots vazios ficam apagados. Estes LEDs indicam o volume controlado, não picos sonoros individuais. A interface explica a preparação e apresenta as mensagens do modo mesmo quando a entrada é recusada. Uma tentativa de atualizar uma aplicação sem sessão deixa de apresentar o valor solicitado como volume aplicado.

Verificação automática: 16 testes do serviço e 32 da interface. A simulação USB agora distingue posições físicas, volumes lógicos, atribuições e latch; verifica movimentos consecutivos sem SetVolume durante o controlo, canais audíveis conservados, encaminhamento isolado, pickup, páginas, LEDs, saída, edição e reposição após erro parcial. Não foi executado áudio de teste. A confirmação acústica e do comportamento físico dos LEDs permanece para a fase final.
