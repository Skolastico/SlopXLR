> Historical Portuguese development note. It records the state at its date; use the English guides for current behavior.

# Estabilidade do GoXLR Mini — 12 outubro 2026

## Verificado

- 26 testes do frontend passaram: contratos, limites por modelo, rascunhos, posições e páginas, validação de ações, erros, encaminhamento e biblioteca de áudio.
- 16 testes Rust passaram: gestos de entrada/saída e avanço/recuo, posse dos botões durante mudanças de modo, persistência de configurações, ordenação das aplicações, mute imediato com USB simulado, ações Play/Stop e recuperação de perfis.
- Mini S210201775DI7 ligado, perfis AXAXAOLA/DEFAULT.
- Pedidos reais de ação play_sample e stop_sample aceites pelo endpoint que executa as ações dos botões, usando um WAV temporário no banco C / Inferior direito. A associação e o ficheiro de teste foram retirados. Não foram substituídas as ações atuais dos botões.
- Reinício real do serviço: perfis, configurações do microfone, mutes, encaminhamento, iluminação, preferências de apresentação, sampler e configurações das páginas/aplicações conservados.
- Atalho GoXLR Utility.lnk na pasta de arranque do utilizador aponta para GoXLR-Personalizado/goxlr-daemon.exe, com --config para config/settings.json e pasta de trabalho correta; o executável existe.

## Falha encontrada e correção

Antes da correção, a primeira leitura USB dos faders do Mini era comparada com uma referência inicial de zero. Era interpretada como movimento do utilizador e substituía volumes restaurados do perfil: neste teste Mic 254 -> 222 e Chat 115 -> 209. Os mutes e os restantes ajustes mantiveram-se.

A primeira leitura do Mini passa a inicializar a referência física sem alterar o perfil. Leituras seguintes só alteram o volume quando a posição física muda. O Full mantém o seu comportamento. O teste cobre volume guardado 130, posição inicial 37, leitura estacionária 37 e movimento posterior para 42; conserva 130 nas duas primeiras leituras e aplica 42 após movimento. Os testes de mute e navegação continuam a passar.

## Confirmação física pendente

Os testes simulados não substituem a pressão dos botões nem a confirmação acústica. Ainda falta confirmar no hardware: entrar/sair do modo de ações, reproduzir/parar com os botões atribuídos, dois/três cliques no Cough para páginas, mute imediato em modo normal e controlo ao mover os faders depois de reiniciar. O reinício completo do Windows não foi executado; apenas o atalho e o reinício do serviço foram verificados.

## Resultado após instalar a correção

Reinício real confirmado: volumes, perfis, mutes, processamento, encaminhamento, iluminação, sampler, ações e aplicações conservaram-se, sem reposição manual dos volumes. App nativa reaberta. Backup do executável anterior, configurações e estados de comparação em ../GoXLR-Personalizado/verificacao-estabilidade-20261012-020042.
