# GoXLR Personalizado

Aplicação Windows x64 personalizada, baseada no GoXLR Utility. Interface em Português de Portugal e inglês, controlos por aplicação, ações nos botões e configuração de áudio.

**0.1.0-beta.1**: testada num GoXLR Mini. O modelo completo ainda precisa de validação física.

## Instalar

Instala o driver oficial GoXLR, encerra outras versões do Utility e abre o instalador. A alternativa portátil é extrair o ZIP e abrir Open.cmd. As configurações são independentes; a primeira abertura permite importar a instalação original sem alterar a origem. A desinstalação conserva os dados.

Idioma e atualizações estão nas Definições. A interface anterior está em Opções avançadas. Enquanto o repositório for privado, descarrega as versões autenticado no GitHub e instala manualmente. A aplicação não inclui credenciais de acesso ao GitHub.

## Construir e publicar

Requer Rust/MSVC, Node.js (testes), NSIS e o driver para testar o dispositivo. Executa node --test tests/*.test.mjs e cargo test -p goxlr-daemon. distribution/Build.ps1 compila o serviço e a janela, gera instalador, ZIP e manifesto de atualização assinado. Guarda a chave RSA privada fora do repositório e passa -SigningKeyPath; a chave pública está em distribution/update-public.xml. Para reutilizar uma janela já compilada: -SkipBuild -UiBinary caminho. -NsisPath permite indicar o compilador NSIS.

O pacote local desta beta reutiliza a janela nativa existente; o serviço foi compilado com as alterações. O workflow compila ambos a partir do código.

Para publicar uma nova versão, altera distribution/release.json e cria uma etiqueta v seguida da mesma versão. O workflow exige o segredo UPDATE_SIGNING_KEY_XML com a chave privada correspondente à chave pública. Publica os cinco ficheiros gerados: instalador, ZIP, update.json, update.json.sig e SHA256SUMS.txt. Sem esse segredo a publicação falha, em vez de gerar pacotes sem assinatura. Releases privadas requerem acesso à conta; o atualizador automático lê apenas releases públicas. A assinatura dos updates não substitui a assinatura comercial Windows dos executáveis.

## Origem e licenças

Mantém o histórico e as licenças do GoXLR-on-Linux/GoXLR-Utility (base 49ab4e4) e inclui a janela GoXLR Utility UI (base e71bb2e) em ui-wrapper. Consulta LICENSE, LICENSE-3RD-PARTY, ui-wrapper/LICENSE e README-UPSTREAM.md.
