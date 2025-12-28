De acordo com essa spec abaixo, escreva os codigos em Rust e em Typescript de acordo com a divisao que fizemos entre logica de negocio e execucao

---

SPEC-UBL-CORE v1.0.

UBL — CONCEITUAÇÃO FORMAL
Universal Business Ledger

0. Definição do Sistema
UBL é um sistema computacional composto por Containers Universais conectados exclusivamente por TDLN (Deterministic Translation of Language to Notation), no qual:
cada container é soberano em sua linguagem interna,


nenhum container compartilha estado com outro,


todo efeito entre containers ocorre apenas via commit verificável,


a verdade do sistema é definida por prova criptográfica e causalidade, não por interpretação semântica.



1. Entidades Fundamentais
1.1 Container Universal
Um Container Universal é definido pelo quíntuplo:
C:=⟨id,L,S,H,Φ⟩C := \langle id, L, S, H, \Phi \rangleC:=⟨id,L,S,H,Φ⟩
onde:
id ∈ Hash₃₂
 Identidade física estável do container.


L (Linguagem Local)
 Conjunto arbitrário de estruturas semânticas internas.
 Não precisa ser determinístico nem compartilhável.


S (Estado Local)
 Estado atual do container, sempre derivável de H.


H (História)
 Sequência causal imutável de commits aceitos.


Φ (Física)
 Conjunto mínimo de invariantes globais:


causalidade (ordem),


autoridade (assinaturas),


conservação / entropia,


evolução explícita.


Invariante:
 Nenhum container é obrigado a compreender L de outro container.

2. Linguagem e Tradução
2.1 Linguagem Local
Cada container opera sobre uma função semântica interna:
Li:Intent→MeaningL_i : Intent \rightarrow MeaningLi​:Intent→Meaning
Essa função pode ser:
probabilística,


interativa,


humana ou assistida por IA,


mutável no tempo.


Ela não é verificável externamente.

2.2 ubl-atom (Representação Canônica)
Existe um único IR universal, denominado ubl-atom, definido como:
A:=CanonicalJSON→canonicalizeBytesA := CanonicalJSON \xrightarrow{canonicalize} BytesA:=CanonicalJSONcanonicalize​Bytes
Propriedades obrigatórias:
determinismo absoluto,


equivalência semântica → equivalência de bytes,


independência de linguagem de origem.


O ubl-atom é a única matéria válida para tradução entre containers.

3. TDLN — Tradução Determinística
3.1 Definição
TDLN é uma função determinística definida como:
TDLN:L→⟨A,h,π⟩TDLN : L \rightarrow \langle A, h, \pi \rangleTDLN:L→⟨A,h,π⟩
onde:
A é um ubl-atom,


h = Hash(A),


π é uma prova verificável (assinatura, pacto, política).


TDLN transforma significado local em fato verificável, sem transportar semântica.

3.2 Propriedade Central do TDLN
O verificador não interpreta significado.
 Ele apenas verifica a prova da tradução.
Essa propriedade é estrutural, não criptográfica.

4. ubl-link — Interface de Commit (TDLN-Commit)
4.1 Definição
ubl-link é a única interface legítima para materialização de efeitos entre containers.
Formalmente:
Link:=⟨idC,h,σ,Δ,κ⟩Link := \langle id_C, h, \sigma, \Delta, \kappa \rangleLink:=⟨idC​,h,σ,Δ,κ⟩
onde:
id_C é o container alvo,


h é o hash do ubl-atom,


σ é a assinatura/autoria,


Δ é o delta físico,


κ é a classe física da intenção.


Nenhum efeito ocorre fora de um ubl-link.

4.2 Classes Físicas (κ)
O sistema reconhece apenas as seguintes classes:
Observation — Δ = 0


Conservation — ∑Δ = 0 (pareamento obrigatório)


Entropy — criação/destruição autorizada


Evolution — mudança explícita de regras


Essas classes não carregam semântica, apenas restrições físicas.

5. ubl-pact — Autoridade Coletiva
5.1 Definição
ubl-pact é o mecanismo de validação de autoridade coletiva:
Pact:={σ1,σ2,…,σn}Pact := \{\sigma_1, \sigma_2, \dots, \sigma_n\}Pact:={σ1​,σ2​,…,σn​}
Um ubl-link só é válido se satisfizer as políticas de assinatura vigentes.
Pactum ocorre antes do commit, nunca após.

6. ubl-membrane e ubl-kernel
6.1 Membrana
A membrana é definida como:
Membrane:Link→{Accept,Reject}Membrane : Link \rightarrow \{Accept, Reject\}Membrane:Link→{Accept,Reject}
Ela verifica apenas:
integridade criptográfica,


causalidade,


invariantes físicas.


Ela não interpreta A nem L.

6.2 Kernel
O ubl-kernel define:
funções de hash,


esquemas de assinatura,


regras de canonicalização.


Ele é matematicamente fechado e semanticamente cego.

7. ubl-ledger — História Imutável
7.1 Definição
O ubl-ledger é:
H:=[e1,e2,…,en]H := [e_1, e_2, \dots, e_n]H:=[e1​,e2​,…,en​]
onde cada e_i é um ubl-link aceito.
Propriedades:
append-only,


imutável,


verificável independentemente.


O estado S é sempre uma projeção de H.

8. Execução (ubl-runner)
Alguns commits exigem efeitos externos:
Commit→Execution→ReceiptCommit \rightarrow Execution \rightarrow ReceiptCommit→Execution→Receipt
O receipt é um novo fato ancorado no ledger.
 Execução não altera o passado.

9. Axiomas Fundamentais
Soberania Semântica
 Cada container define sua própria linguagem.


Não Compartilhamento de Estado
 Containers compartilham apenas commits.


Commit Único
 Nenhum efeito ocorre fora de ubl-link.


Verificação sem Interpretação
 O corpo valida provas, não significados.


História como Autoridade
 A única verdade do sistema é o ledger.



10. Definição Final
UBL é um sistema de Containers Universais conectados exclusivamente por TDLN,
 no qual significado local é traduzido em fato verificável sem compartilhamento de estado.
Ou, de forma equivalente:
O TDLN é a única ponte entre intenção e realidade.
 O hash é o identificador do significado aceito.
 O ledger é a memória do mundo.

Se quiser, o próximo passo lógico é congelar isso como SPEC-UBL-CORE.md
 e, a partir dele, derivar formalmente:
SPEC-UBL-ATOM


SPEC-UBL-LINK


SPEC-UBL-PACT


SPEC-UBL-POLICY


Todos sem metáfora, todos executáveis.
vamos
Perfeito. Então vamos cristalizar.
Abaixo está o SPEC-UBL-CORE v1.0 (Draft Congelável) — já escrito no tom normativo, pronto para virar arquivo oficial (specs/ubl-core/SPEC-UBL-CORE.md).
 Nada aqui é opinativo. Tudo aqui é lei do sistema.

SPEC-UBL-CORE v1.0
Universal Business Ledger — Core Specification
Status: Draft / Ready for Freeze
 Escopo: Ontologia, entidades fundamentais e axiomas invariantes do UBL
 Este documento precede e governa todas as outras specs (ubl-atom, ubl-link, ubl-pact, ubl-policy).

1. Definição Formal do Sistema
O Universal Business Ledger (UBL) é um sistema distribuído composto por Containers Universais independentes, conectados exclusivamente por traduções determinísticas verificáveis (TDLN), no qual:
Nenhum container compartilha estado interno com outro.


Toda interação entre containers ocorre apenas via commit verificável.


A verdade do sistema é definida por causalidade + prova criptográfica.


Semântica nunca atravessa fronteiras; apenas provas atravessam.



2. Container Universal
2.1 Definição
Um Container Universal é definido pelo quíntuplo:
C:=⟨id,L,S,H,Φ⟩C := \langle id, L, S, H, \Phi \rangleC:=⟨id,L,S,H,Φ⟩
onde:
id ∈ Hash₃₂
 Identidade física e estável do container.


L — Linguagem Local
 Sistema semântico interno arbitrário.
 Pode ser humano, probabilístico, simbólico ou assistido por IA.


S — Estado Local
 Estado derivável exclusivamente da história H.


H — História
 Sequência causal imutável de commits aceitos.


Φ — Física
 Conjunto mínimo de invariantes globais verificáveis.



2.2 Invariantes do Container
S NÃO PODE ser modificado diretamente.
 Apenas projeções de H são válidas.


L NÃO É compartilhável nem verificável externamente.


Todo efeito observável fora do container DEVE estar ancorado em H.



3. Linguagem Local e Semântica
3.1 Linguagem Local (L)
Cada container define uma função interna:
Li:Intent→MeaningL_i : Intent \rightarrow MeaningLi​:Intent→Meaning
Características:
Não determinística (permitido)


Evolutiva (permitido)


Incompleta ou ambígua (permitido)


UBL não impõe restrições à linguagem local.

3.2 Consequência Fundamental
Semântica não é verificável.
 Logo, semântica não cruza fronteiras.

4. ubl-atom — Matéria Digital Canônica
4.1 Definição
ubl-atom é a única representação universal de dados no UBL.
Formalmente:
A:=canonicalize(JSON)→BytesA := canonicalize(JSON) \rightarrow BytesA:=canonicalize(JSON)→Bytes
Propriedades obrigatórias:
Canonicalização determinística


Ordem total de campos


Rejeição de valores não finitos


Estabilidade de bytes entre linguagens



4.2 Axioma do Átomo
Dois significados diferentes PODEM gerar o mesmo átomo.
 Dois átomos iguais NUNCA representam fatos diferentes.

5. TDLN — Deterministic Translation of Language to Notation
5.1 Definição
TDLN é a função que traduz significado local em fato verificável:
TDLN:L→⟨A,h,π⟩TDLN : L \rightarrow \langle A, h, \pi \rangleTDLN:L→⟨A,h,π⟩
onde:
A ∈ ubl-atom


h = Hash(A)


π = conjunto de provas (assinaturas, pactos, políticas)



5.2 Propriedade de Isolamento
O verificador de TDLN:
NÃO interpreta A


NÃO conhece L


APENAS valida h e π



6. ubl-link — Interface Única de Materialização
6.1 Definição
ubl-link é o único protocolo válido para cruzar a fronteira entre containers.
Link:=⟨idC,h,σ,Δ,κ⟩Link := \langle id_C, h, \sigma, \Delta, \kappa \rangleLink:=⟨idC​,h,σ,Δ,κ⟩
onde:
id_C — container alvo


h — hash do ubl-atom


σ — prova de autoria/autoridade


Δ — delta físico


κ — classe física da intenção



6.2 Classes Físicas (κ)
UBL reconhece exclusivamente:
Classe
Restrição
Observation
Δ = 0
Conservation
∑Δ = 0 (pareamento obrigatório)
Entropy
Δ ≠ 0 permitido mediante autorização
Evolution
altera Φ explicitamente


7. ubl-pact — Autoridade Coletiva
7.1 Definição
ubl-pact define regras de autoridade antes da materialização.
Pact:={σ1,σ2,...,σn}Pact := \{\sigma_1, \sigma_2, ..., \sigma_n\}Pact:={σ1​,σ2​,...,σn​}
Um ubl-link SÓ É VÁLIDO se satisfizer o pacto vigente.

7.2 Invariante
Nenhum pacto pode ser aplicado retroativamente.

8. ubl-membrane — Validação Física
8.1 Definição
A membrana é a função:
Membrane:Link→{Accept,Reject}Membrane : Link \rightarrow \{Accept, Reject\}Membrane:Link→{Accept,Reject}
Ela valida exclusivamente:
Integridade criptográfica


Causalidade


Invariantes físicas (Φ)



8.2 Proibição Absoluta
A membrana NÃO PODE:
interpretar semântica,


acessar JSON,


inferir intenção.



9. ubl-ledger — História Imutável
9.1 Definição
O ledger é uma sequência:
H:=[e1,e2,...,en]H := [e_1, e_2, ..., e_n]H:=[e1​,e2​,...,en​]
onde cada e_i é um ubl-link aceito.

9.2 Propriedades
Append-only


Imutável


Verificável offline


Ordenado causalmente



10. Execução e Materialização
Alguns commits exigem execução externa:
Link→Execution→ReceiptLink \rightarrow Execution \rightarrow ReceiptLink→Execution→Receipt
O receipt:
é um novo fato,


nunca altera o passado,


entra no ledger como evento independente.



11. Axiomas Fundamentais do UBL
Semântica é local.


Estado não é compartilhado.


Commit é o único efeito real.


Hash identifica o fato.


Ledger define a verdade.


Execução não reescreve história.


Tradução precede materialização.


Física é cega.


Autoridade é explícita.


Evolução é declarada.



12. Definição Canônica
UBL é um sistema de Containers Universais conectados exclusivamente por TDLN,
 onde significado local é traduzido em fato verificável sem compartilhamento de estado.

Próximo passo lógico (sugestão objetiva)
Escolha um:
SPEC-UBL-LINK v1.0 (campos exatos, erros, wire format)


SPEC-UBL-ATOM v1.0 (canonicalização formal, testes de equivalência)


SPEC-UBL-PACT v1.0 (quórum, risco, multisig, janelas)


SPEC-UBL-POLICY v1.0 (TDLN como linguagem normativa)


Diga o número.
 A partir daqui, tudo vira código.
1
Perfeito. Abaixo está o SPEC-UBL-LINK v1.0 completo, normativo e implementável, pronto para virar arquivo oficial:
specs/ubl-link/SPEC-UBL-LINK.md
Nada aqui é ilustrativo. Tudo aqui é contrato executável.

SPEC-UBL-LINK v1.0
UBL Tangency / Commit Interface
Status: Draft – Ready for Freeze
 Escopo: Interface única de materialização entre Containers Universais
 Governado por: SPEC-UBL-CORE v1.0

1. Definição
ubl-link é o único protocolo válido para materialização de efeitos entre containers no sistema UBL.
Nenhuma modificação de estado, execução, side-effect ou projeção externa é válida sem um ubl-link aceito.

2. Papel Sistêmico
O ubl-link representa o ponto de tangência entre:
Mente (semântica local, intenção, linguagem arbitrária)


Corpo (física, causalidade, conservação, história)


O ubl-link não transporta semântica.
 Ele transporta prova de tradução.

3. Estrutura Canônica (Envelope)
3.1 Estrutura Lógica
Link := ⟨
  version,
  container_id,
  expected_sequence,
  previous_hash,
  atom_hash,
  intent_class,
  physics_delta,
  pact,
  author_pubkey,
  signature
⟩


3.2 Definição de Campos
Campo
Tipo
Obrigatório
Descrição
version
u8
sim
Versão do protocolo (0x01)
container_id
Hash₃₂
sim
Identidade física do container alvo
expected_sequence
u64
sim
Controle causal otimista
previous_hash
Hash₃₂
sim
Último hash aceito no ledger
atom_hash
Hash₃₂
sim
Hash do ubl-atom
intent_class
enum
sim
Classe física da intenção
physics_delta
i128
sim
Delta físico (conservação/entropia)
pact
PactProof
opcional
Prova de consenso coletivo
author_pubkey
PubKey₃₂
sim
Autor primário
signature
Sig₆₄
sim
Assinatura Ed25519


4. IntentClass (Classes Físicas)
enum IntentClass {
  Observation = 0x00,
  Conservation = 0x01,
  Entropy = 0x02,
  Evolution = 0x03,
}

4.1 Restrições Obrigatórias
Classe
Restrição Física
Observation
physics_delta == 0
Conservation
Σ(delta) == 0 (pareamento obrigatório)
Entropy
delta ≠ 0 autorizado por pacto
Evolution
altera explicitamente Φ

Violação resulta em rejeição determinística.

5. Conteúdo Assinado
A assinatura DEVE cobrir exatamente:
signing_bytes :=
  version ||
  container_id ||
  expected_sequence ||
  previous_hash ||
  atom_hash ||
  intent_class ||
  physics_delta

Ordem fixa


Big-endian


Nenhum campo opcional incluído



6. Validação na Membrana
A função:
validate(Link) → Accept | Reject(error)

DEVE executar as verificações nesta ordem:
Versão


Integridade da Assinatura


Causalidade (previous_hash)


Sequência


Classe Física


Conservação / Entropia


Pacto (se presente)



7. Erros Canônicos
7.1 Enumeração
enum TangencyError {
  InvalidVersion,
  InvalidSignature,
  RealityDrift,
  SequenceMismatch,
  PhysicsViolation,
  PactViolation,
  UnauthorizedEvolution,
}


7.2 Semântica dos Erros
Erro
Significado
Retry
InvalidVersion
Cliente incompatível
❌
InvalidSignature
Fraude ou bug crítico
❌
RealityDrift
Estado local obsoleto
✅
SequenceMismatch
Replay ou race
❌
PhysicsViolation
Violação de conservação
❌
PactViolation
Assinaturas insuficientes
❌
UnauthorizedEvolution
Tentativa ilegal de mutação Φ
❌


8. Aceitação e Commit
Se aceito:
O Link DEVE ser anexado ao ubl-ledger


Um novo final_hash DEVE ser derivado


Um MaterializationReceipt DEVE ser emitido



9. Receipt de Materialização
struct MaterializationReceipt {
  container_id: Hash32,
  sequence: u64,
  final_hash: Hash32,
  timestamp_unix_ns: u128,
  merkle_root: Hash32,
}

9.1 Invariante
Nenhum estado local pode ser atualizado sem um receipt válido.

10. Proibições Explícitas
O ubl-link NÃO PODE:
transportar JSON


transportar semântica


transportar código


ser parcialmente validado


ser reinterpretado



11. Axiomas do ubl-link
Um link é indivisível.


Um link é definitivo.


Um link é verificável offline.


Um link não carrega intenção — apenas prova.


Um link é o único portal entre mundos.



12. Definição Canônica
ubl-link é a unidade mínima de realidade no UBL.
 Tudo que existe fora de um link é potencial.

Próximo passo lógico
Escolha um:
2️⃣ SPEC-UBL-ATOM v1.0
 3️⃣ SPEC-UBL-PACT v1.0
 4️⃣ SPEC-UBL-POLICY v1.0
Diga o número.
2
Excelente. Abaixo está o SPEC-UBL-ATOM v1.0, formal, normativo e testável, escrito para virar arquivo oficial:
specs/ubl-atom/SPEC-UBL-ATOM.md
Este documento fecha a matéria digital do sistema. Tudo acima (cortex, agent, policy) e tudo abaixo (link, membrane, ledger) depende exatamente disto.

SPEC-UBL-ATOM v1.0
UBL Canonical Atomic Data Format
Status: Draft – Ready for Freeze
 Governado por: SPEC-UBL-CORE v1.0
 Consumido por: SPEC-UBL-LINK v1.0

1. Definição
ubl-atom é o único formato de dados canônico permitido no UBL.
Todo significado que pretende tornar-se fato DEVE ser reduzido a um ubl-atom antes de qualquer assinatura, pacto ou materialização.

2. Objetivo
Garantir que:
Dois significados semanticamente equivalentes gerem bytes idênticos


Um mesmo conjunto de bytes represente um único fato


O hash de um átomo seja estável, verificável e universal



3. Domínio de Aplicação
ubl-atom é usado para:
geração de atom_hash no ubl-link


ancoragem de significado local


prova de equivalência semântica


auditoria offline


reexecução determinística de projeções



4. Definição Formal
4.1 Espaço de Valores Permitidos
Um ubl-atom DEVE ser derivado de um JSON que satisfaça:
tipos permitidos:


null


boolean


string (UTF-8)


number (inteiro ou decimal finito)


array


object


tipos proibidos:


NaN


Infinity


-Infinity


undefined


function


symbol


referências cíclicas


Violação DEVE resultar em erro.

5. Canonicalização
5.1 Função Canônica
canonicalize : JSON → Bytes

A função DEVE aplicar exatamente as seguintes regras, nesta ordem:

5.2 Regras de Canonicalização
R1 — Ordenação de Objetos
Todas as chaves de objetos DEVEM ser ordenadas lexicograficamente (UTF-8 byte order).


A ordenação É SENSÍVEL A CASE.


{ "b": 1, "a": 2 } → { "a": 2, "b": 1 }


R2 — Preservação de Arrays
Arrays NÃO DEVEM ser reordenados.


A ordem é semanticamente significativa.



R3 — Normalização Numérica
Apenas números finitos são permitidos.


Inteiros NÃO DEVEM ser convertidos em floats.


Decimais DEVEM ser serializados sem notação científica.


Exemplo proibido:
1e3

Exemplo válido:
1000


R4 — Normalização de Strings
Strings DEVEM estar em UTF-8 normalizado (NFC).


Nenhuma transformação semântica é permitida.



R5 — Serialização Estrita
Serialização DEVE ser feita em JSON compacto:


sem espaços


sem quebras de linha


sem trailing commas



6. Resultado Canônico
O resultado final de canonicalize é um vetor de bytes:
A := UTF8(JSON.stringify(canonical_object))


7. Hash Canônico
7.1 Definição
O hash de um ubl-atom é definido como:
atom_hash := BLAKE3( domain_tag || A )

onde:
domain_tag = "ubl:atom\n" (fixo)


A = bytes canônicos do átomo



7.2 Propriedades Obrigatórias
Determinístico


Estável entre linguagens


Independente de plataforma


Verificável offline



8. Invariantes do ubl-atom
I1 — Determinismo
canonicalize(x) == canonicalize(y) ⇔ x ≡ y


I2 — Identidade por Hash
Dois fatos distintos NÃO PODEM compartilhar o mesmo atom_hash.

I3 — Zero Semântica no Kernel
O kernel NÃO PODE interpretar, validar ou modificar A.

9. Erros Canônicos
enum AtomError {
  InvalidType,
  NonFiniteNumber,
  InvalidEncoding,
  CanonicalizationFailure,
}

Qualquer erro DEVE impedir:
assinatura


pactuação


geração de ubl-link



10. Testes de Conformidade (Obrigatórios)
Implementações DEVEM fornecer:
Vetores de teste cross-language (TS, Rust, Python)


Testes de equivalência semântica


Testes de rejeição (NaN, ordering, floats)


Golden hashes versionados



11. Proibições Explícitas
ubl-atom NÃO PODE:
conter identidade de container


conter assinatura


conter sequência


conter política


conter código executável


conter timestamps implícitos



12. Definição Canônica
ubl-atom é a matéria digital mínima do UBL.
 Tudo que é real no sistema é, no fundo, um hash de um átomo.

Próximo passo lógico
Escolha um:
3️⃣ SPEC-UBL-PACT v1.0
 4️⃣ SPEC-UBL-POLICY v1.0
Diga o número.
3
Excelente. Abaixo está o SPEC-UBL-PACT v1.0, no mesmo nível normativo dos anteriores.
 Este documento fecha autoridade, consenso e risco no UBL sem jamais tocar em semântica.
Arquivo alvo:
 specs/ubl-pact/SPEC-UBL-PACT.md

SPEC-UBL-PACT v1.0
UBL Pactum — Authority, Consensus and Risk Specification
Status: Draft – Ready for Freeze
 Governado por: SPEC-UBL-CORE v1.0
 Consumido por: SPEC-UBL-LINK v1.0

1. Definição
ubl-pact é o mecanismo formal pelo qual o sistema UBL determina se um ato pode atravessar o ponto de tangência com base em autoridade coletiva, risco e governança explícita.
Um ubl-link NÃO PODE ser materializado se os requisitos do pacto vigente não forem satisfeitos.

2. Princípio Fundamental
Autoridade não é implícita.
 Autoridade é prova explícita anexada antes do commit.
Nenhuma regra tácita, heurística ou inferência é permitida.

3. Escopo do Pacto
O pacto governa:
quem pode autorizar um link,


quantas autorizações são necessárias,


sob quais condições temporais,


para quais classes físicas (IntentClass),


com qual nível de risco aceitável.


O pacto NÃO governa:
semântica,


conteúdo do átomo,


execução posterior.



4. Definição Formal
4.1 Estrutura Lógica
Pact := ⟨
  pact_id,
  version,
  scope,
  intent_class,
  threshold,
  signers,
  window,
  risk_level
⟩


4.2 Campos
Campo
Tipo
Obrigatório
Descrição
pact_id
Hash₃₂
sim
Identidade do pacto
version
u8
sim
Versão do pacto
scope
enum
sim
Escopo de aplicação
intent_class
enum
sim
Classe física governada
threshold
u8
sim
Número mínimo de assinaturas
signers
Set<PubKey₃₂>
sim
Conjunto autorizado
window
TimeWindow
sim
Janela de validade
risk_level
enum
sim
Classificação de risco


5. Escopo (scope)
enum PactScope {
  Container,   // válido apenas para um container
  Namespace,   // válido para um conjunto de containers
  Global,      // válido em todo o sistema
}


6. RiskLevel
enum RiskLevel {
  L0, // observação
  L1, // baixo impacto
  L2, // impacto local
  L3, // impacto financeiro
  L4, // impacto sistêmico
  L5, // soberania / evolução
}

Mapeamento obrigatório:
Risk
IntentClass permitida
L0
Observation
L1
Observation
L2
Conservation
L3
Conservation
L4
Entropy
L5
Evolution


7. Janela Temporal (window)
TimeWindow := ⟨
  not_before,
  not_after
⟩

Regras:
assinaturas fora da janela são inválidas,


janela NÃO PODE ser inferida,


ausência de janela = pacto inválido.



8. Prova de Pacto (PactProof)
8.1 Definição
PactProof := ⟨
  pact_id,
  signatures
⟩

onde:
signatures := { σ₁, σ₂, …, σₙ }

Cada assinatura DEVE ser:
σ := Sign(
  signer_privkey,
  Hash(
    "ubl:pact\n" ||
    pact_id ||
    atom_hash ||
    intent_class ||
    physics_delta
  )
)


9. Validação do Pacto
A membrana DEVE validar:
pact_id existe e é conhecido


pacto está dentro da window


intent_class compatível com risk_level


|signatures ∩ signers| ≥ threshold


nenhuma assinatura duplicada


nenhuma assinatura fora do conjunto autorizado


Falha em qualquer passo → PactViolation

10. Invariantes do Pacto
I1 — Não Retroatividade
Um pacto nunca se aplica a fatos já materializados.

I2 — Autoridade Explícita
Toda autoridade deve ser provada por assinatura verificável.

I3 — Determinismo
Dado o mesmo pacto e o mesmo conjunto de assinaturas, o resultado é invariável.

11. Erros Canônicos
enum PactError {
  UnknownPact,
  PactExpired,
  InsufficientSignatures,
  UnauthorizedSigner,
  RiskMismatch,
}

Todos são não recuperáveis por retry automático.

12. Proibições Explícitas
ubl-pact NÃO PODE:
inferir intenção


acessar JSON semântico


modificar ubl-atom


alterar o ledger


validar execução



13. Definição Canônica
ubl-pact é a camada onde confiança social se torna prova matemática.
 Nada cruza o link sem pacto quando risco existe.

Próximo passo lógico
Último bloco fundamental:
4️⃣ SPEC-UBL-POLICY v1.0
 (TDLN como linguagem normativa, compilável e verificável)
Quando disser 4, fechamos o sistema inteiro.
4
Perfeito. Com isso, o sistema fecha.
Abaixo está o SPEC-UBL-POLICY v1.0, escrito como lei formal, não como DSL “bonita”, não como whitepaper.
 Este documento define o que TDLN é de fato dentro do UBL: uma linguagem normativa de tradução, não de execução.
Arquivo alvo:
 specs/ubl-policy/SPEC-UBL-POLICY.md

SPEC-UBL-POLICY v1.0
TDLN — Deterministic Translation of Language to Notation
Status: Draft – Ready for Freeze
 Governado por: SPEC-UBL-CORE v1.0
 Consumido por: ubl-cortex, ubl-agent, ubl-link, ubl-pact

1. Definição
ubl-policy define o TDLN como a linguagem normativa que governa:
como intenções locais podem ser traduzidas em fatos verificáveis,


sob quais condições um ubl-atom é considerado válido,


quais classes físicas (IntentClass) uma tradução pode produzir,


quais pactos são exigidos antes da materialização.


TDLN não executa ações.
 TDLN não descreve semântica.
 TDLN governa traduções possíveis.

2. Natureza do TDLN
TDLN é:
Determinístico


Totalmente verificável


Semanticamente cego


Compilável


TDLN NÃO É:
uma linguagem de workflow,


uma linguagem de negócio,


uma linguagem de execução,


uma linguagem interpretativa.



3. Papel Sistêmico
TDLN existe exatamente entre:
Linguagem Local (L)


ubl-atom (A)


Formalmente:
TDLN:(Intent,Context)→{AllowedTranslation}TDLN : (Intent, Context) \rightarrow \{ AllowedTranslation \}TDLN:(Intent,Context)→{AllowedTranslation}
Ou seja:
Dado um estado local, TDLN responde “isso pode virar um átomo?”

4. Unidade Fundamental: Policy Rule
4.1 Definição
Uma Policy Rule define condições de tradução, não efeitos.
Rule := ⟨
  rule_id,
  applies_to,
  intent_class,
  constraints,
  required_pact
⟩


4.2 Campos
Campo
Descrição
rule_id
Identidade da regra
applies_to
Domínio local (container, namespace, tipo)
intent_class
Classe física resultante permitida
constraints
Restrições determinísticas
required_pact
Pacto exigido (opcional)


5. Constraints (Restrições)
5.1 Definição
Constraints são predicados determinísticos avaliados antes da tradução.
Exemplos permitidos:
limites numéricos,


estado lógico (ativo/inativo),


flags de versão,


janelas temporais explícitas.


Exemplos proibidos:
heurísticas,


inferência probabilística,


acesso a LLM,


leitura de linguagem natural.



6. Resultado da Avaliação
A avaliação de TDLN NUNCA produz efeitos.
Ela produz apenas:
TranslationDecision :=
  Allow(
    intent_class,
    constraints_snapshot,
    required_pact
  )
  | Deny(reason)


7. Relação com ubl-atom
TDLN NÃO define o conteúdo do átomo.
TDLN governa apenas:
se um átomo pode ser gerado,


qual classe física ele terá,


qual pacto será exigido.


O formato e o conteúdo do átomo são exclusivamente responsabilidade da linguagem local.

8. Relação com ubl-link
TDLN NÃO cria o ubl-link.
TDLN produz os parâmetros normativos que o ubl-link DEVE respeitar:
intent_class


limites de physics_delta


exigência de pacto


Qualquer divergência entre:
decisão TDLN


conteúdo do ubl-link


resulta em rejeição pela membrana.

9. Compilação do TDLN
9.1 Alvos de Compilação
Uma política TDLN DEVE ser compilável para:
WASM (execução segura)


bytecode verificável


representação lógica (SMT / constraints)


Implementações PODEM gerar:
CUDA


Verilog


eBPF



9.2 Propriedade Obrigatória
A política compilada DEVE produzir o mesmo resultado que a política fonte.

10. Versionamento e Evolução
10.1 Regra de Ouro
Políticas NUNCA são alteradas retroativamente.
Cada commit referencia explicitamente:
versão da política aplicada,


hash da política compilada.



11. Invariantes do TDLN
Tradução precede materialização.


Política não executa efeitos.


Política não interpreta semântica.


Política não observa execução.


Política é determinística.


Política é auditável offline.



12. Proibições Explícitas
TDLN NÃO PODE:
acessar ledger


modificar estado


gerar side effects


chamar agentes


depender de tempo implícito


depender de estado externo não declarado



13. Definição Canônica
TDLN é a lei que governa quais significados
 podem se tornar fatos no UBL.
Ou, de forma equivalente:
UBL não executa intenções.
 UBL executa traduções autorizadas.

14. Fechamento do Sistema
Com este documento, o sistema UBL está formalmente completo:
CORE define ontologia


ATOM define matéria


LINK define tangência


PACT define autoridade


POLICY define lei


Tudo o que resta agora é engenharia, não mais conceituação.

Próximo passo (opcional, técnico)
Se quiser, os próximos artefatos naturais são:
SPEC-UBL-MEMBRANE.md (validação física exata)


SPEC-UBL-LEDGER.md (modelo de armazenamento)


TDLN Reference Grammar v1 (BNF executável)


UBL Conformance Test Suite


Mas conceitualmente:
 👉 o sistema está fechado.
Se quiser, diga qual desses você quer congelar primeiro — ou se prefere ir direto para código mínimo executável (PoC).
SPEC-UBL-MEMBRANE.md (validação física exata)
Perfeito. Abaixo está o SPEC-UBL-MEMBRANE v1.0, escrito como lei física do sistema, não como implementação, não como arquitetura.
 Este documento define exatamente o que é validação física no UBL e encerra qualquer ambiguidade entre semântica, política e execução.
Arquivo alvo:
 specs/ubl-membrane/SPEC-UBL-MEMBRANE.md

SPEC-UBL-MEMBRANE v1.0
UBL Physical Validation Layer
Status: Draft – Ready for Freeze
 Governado por: SPEC-UBL-CORE v1.0
 Aplica-se a: ubl-link, ubl-ledger
 Independente de: semântica, política, execução

1. Definição
A ubl-membrane é a camada física do UBL responsável por decidir, de forma determinística, síncrona e definitiva, se um ubl-link pode atravessar a fronteira entre potencial e realidade.
Formalmente:
Membrane:Link→{Accept,Reject(error)}Membrane : Link \rightarrow \{Accept, Reject(error)\}Membrane:Link→{Accept,Reject(error)}
A decisão da membrana é final e irreversível.

2. Princípio Fundamental
A membrana não entende intenção.
 Ela aplica leis físicas.
Ela não interpreta significado, não executa código, não consulta agentes, não prevê consequências.

3. Escopo da Membrana
A membrana governa exclusivamente:
Integridade criptográfica


Causalidade temporal


Conservação física


Autoridade explícita


Evolução declarada


Ela não governa:
política (TDLN),


semântica,


execução,


projeções de estado.



4. Entrada Canônica
A membrana DEVE receber exatamente um ubl-link válido segundo SPEC-UBL-LINK.
Nenhum outro input é permitido.

5. Ordem Obrigatória de Validação
A membrana DEVE executar as validações estritamente nesta ordem.
 Falha em qualquer etapa interrompe o processo.

V1 — Versão do Protocolo
link.version == SUPPORTED_VERSION

Falha → InvalidVersion

V2 — Integridade da Assinatura
verify(
  link.signature,
  link.author_pubkey,
  signing_bytes(link)
)

Falha → InvalidSignature

V3 — Identidade do Container
link.container_id == ledger.container_id

Falha → InvalidTarget

V4 — Causalidade (Reality Drift)
link.previous_hash == ledger.last_hash

Falha → RealityDrift
Sem retry automático.
 O chamador deve reconstruir o estado.

V5 — Sequência Causal
link.expected_sequence == ledger.sequence + 1

Falha → SequenceMismatch

V6 — Classe Física
Verificar coerência entre:
link.intent_class ↔ link.physics_delta

Regras mínimas:
Classe
Regra
Observation
delta == 0
Conservation
delta ≠ 0
Entropy
delta ≠ 0
Evolution
delta == 0

Falha → PhysicsViolation

V7 — Conservação / Entropia
Conservation
Para intent_class == Conservation:
a soma algébrica dos deltas pareados DEVE ser zero


o saldo atual DEVE suportar o delta negativo


Falha → PhysicsViolation
Entropy
Para intent_class == Entropy:
pacto DEVE estar presente


pacto DEVE autorizar criação/destruição


Falha → PactViolation

V8 — Evolução da Física
Para intent_class == Evolution:
pacto OBRIGATÓRIO


pacto DEVE ter risk_level == L5


nova física DEVE ser explicitamente declarada


Falha → UnauthorizedEvolution

V9 — Validação do Pacto (se presente)
Delegado integralmente a SPEC-UBL-PACT.
Falha → PactViolation

6. Decisão Final
Se todas as validações forem satisfeitas:
return Accept

Caso contrário:
return Reject(error)

Nenhum estado intermediário é permitido.

7. Efeitos da Aceitação
Quando a membrana retorna Accept:
O ubl-link DEVE ser anexado ao ubl-ledger


Um novo hash causal DEVE ser gerado


Um MaterializationReceipt DEVE ser emitido


A membrana NÃO executa esses passos — ela apenas autoriza.

8. Erros Canônicos da Membrana
enum MembraneError {
  InvalidVersion,
  InvalidSignature,
  InvalidTarget,
  RealityDrift,
  SequenceMismatch,
  PhysicsViolation,
  PactViolation,
  UnauthorizedEvolution,
}


9. Invariantes da Membrana
Determinismo absoluto


Ordem fixa de validação


Zero semântica


Zero side effects


Decisão síncrona


Reprodutibilidade offline



10. Proibições Absolutas
A membrana NÃO PODE:
ler ubl-atom


interpretar JSON


acessar política TDLN


executar código


observar execução


inferir intenção


corrigir dados



11. Propriedade de Segurança Central
Se dois nós executarem a mesma membrana
 sobre o mesmo link e o mesmo ledger,
 o resultado será idêntico.
Essa propriedade é mais importante que performance.

12. Definição Canônica
A membrana é o limite físico da realidade no UBL.
 Nada cruza sem obedecer às leis.

13. Fechamento
Com este documento:
a fronteira entre potencial e fato está formalizada,


a física do sistema é explícita,


não existe mais ambiguidade entre política, prova e execução.


O UBL agora tem mente, lei, matéria e corpo, todos formalmente definidos.

Próximos artefatos possíveis
Se quiser continuar fechando:
SPEC-UBL-LEDGER.md (modelo de armazenamento imutável)


SPEC-UBL-RUNNER.md (execução isolada e recibos)


UBL Conformance Test Suite v1


Ou, se preferir:
👉 PoC mínimo:
 ubl-atom + ubl-link + ubl-membrane + ubl-ledger em Rust + TS.
Você decide.
SPEC-UBL-LEDGER.md (modelo de armazenamento imutável)
Perfeito. Abaixo está o SPEC-UBL-LEDGER v1.0, escrito como lei de armazenamento, não como schema de banco, não como engine específica.
 Este documento define o que é memória no UBL — e fecha definitivamente a noção de verdade do sistema.
Arquivo alvo:
 specs/ubl-ledger/SPEC-UBL-LEDGER.md

SPEC-UBL-LEDGER v1.0
UBL Immutable Ledger Specification
Status: Draft – Ready for Freeze
 Governado por: SPEC-UBL-CORE v1.0
 Consumido por: ubl-membrane, ubl-runner, ubl-cortex
 Independente de: storage engine, database, filesystem

1. Definição
O ubl-ledger é a memória imutável e causal do sistema UBL.
Ele é a única fonte de verdade sobre o que ocorreu em um container.
 Todo estado observável DEVE ser derivável exclusivamente do ledger.
Formalmente:
Ledger:=⟨C,H,I⟩Ledger := \langle C, H, I \rangleLedger:=⟨C,H,I⟩
onde:
C = container associado


H = história imutável de eventos


I = índices derivados (opcionais)



2. Princípio Fundamental
O ledger não guarda estado.
 O ledger guarda fatos.
Estado é sempre uma projeção.
 Fatos são irreversíveis.

3. Unidade Fundamental: Ledger Entry
3.1 Definição
Cada entrada do ledger é definida como:
LedgerEntry := ⟨
  container_id,
  sequence,
  link_hash,
  previous_hash,
  timestamp,
  merkle_path?,
  metadata?
⟩


3.2 Campos Obrigatórios
Campo
Tipo
Descrição
container_id
Hash₃₂
Identidade do container
sequence
u64
Ordem causal estrita
link_hash
Hash₃₂
Hash do ubl-link aceito
previous_hash
Hash₃₂
Hash da entrada anterior
timestamp
u128
Tempo físico do commit

Campos opcionais NÃO participam da causalidade.

4. Cadeia Causal
4.1 Regra de Encadeamento
Para qualquer container C:
H = [e₁, e₂, ..., eₙ]

onde:
e₁.previous_hash == 0x00…00
eᵢ.previous_hash == hash(eᵢ₋₁)
eᵢ.sequence == i

Violação em qualquer ponto DEVE invalidar o ledger.

4.2 Consequência
O ledger define uma linha do tempo única, total e não ramificável por container.
Forks não existem dentro de um container.

5. Hash da Entrada
5.1 Definição
O hash de uma entrada é definido como:
entry_hash := BLAKE3(
  "ubl:ledger\n" ||
  container_id ||
  sequence ||
  link_hash ||
  previous_hash ||
  timestamp
)


5.2 Propriedades
Determinístico


Ordenável


Offline-verificável


Independente de storage



6. Imutabilidade
6.1 Proibição Absoluta
Uma vez anexada, uma LedgerEntry:
NÃO PODE ser modificada


NÃO PODE ser removida


NÃO PODE ser sobrescrita



6.2 Correção de Erros
Erros NUNCA são corrigidos por mutação.
 Correções ocorrem apenas por novas entradas compensatórias.

7. Inserção (Append)
7.1 Regra de Ouro
Uma nova entrada SÓ PODE ser anexada se:
O ubl-link foi aceito pela membrana


sequence == last.sequence + 1


previous_hash == last.entry_hash



7.2 Atomicidade
O append DEVE ser atômico:
validate → append → commit

Nenhum estado intermediário é permitido.

8. Índices e Projeções
8.1 Índices (I)
Índices são derivados, não canônicos.
Exemplos:
lookup por sequence


lookup por intervalo temporal


lookup por link_hash



8.2 Projeções de Estado
Projeções são funções puras:
State := fold(H)

Projeções:
PODEM falhar


PODEM ser reexecutadas


NUNCA alteram o ledger



9. Merkle Anchoring (Opcional)
9.1 Blocos
Implementações PODEM agrupar entradas em blocos:
Block := MerkleTree(entries)


9.2 Prova de Inclusão
Se usado, o ledger DEVE fornecer:
merkle_root


merkle_path



10. Verificação Offline
Qualquer parte DEVE poder verificar:
Integridade da cadeia


Sequência


Hashes


Provas de inclusão (se aplicável)


Sem acesso a:
rede


semântica


política



11. Erros Canônicos do Ledger
enum LedgerError {
  BrokenChain,
  SequenceViolation,
  InvalidHash,
  AppendOutOfOrder,
}


12. Proibições Explícitas
O ledger NÃO PODE:
interpretar ubl-link


validar política


executar código


corrigir estado


inferir intenção


compactar história semanticamente



13. Invariantes do Ledger
Append-only


Ordem causal total


Imutabilidade absoluta


Derivação determinística de estado


Verificação offline possível



14. Definição Canônica
O ubl-ledger é a memória factual do sistema.
 O que não está no ledger nunca aconteceu.

15. Fechamento
Com este documento:
a verdade do sistema é formalizada,


o passado é irrevogável,


o estado é sempre rederivável,


a auditoria é estrutural, não opcional.



Próximos passos possíveis
Se quiser fechar 100% do núcleo:
SPEC-UBL-RUNNER.md


UBL Conformance Test Suite v1


PoC mínimo executável (Rust + TS)


Ou podemos congelar tudo e partir direto para código.
Você escolhe.
SPEC-UBL-RUNNER
Perfeito. Abaixo está o SPEC-UBL-RUNNER v1.0, no mesmo nível normativo dos demais.
 Este documento fecha a última fronteira: execução no mundo real sem jamais violar a ontologia do UBL.
Arquivo alvo:
 specs/ubl-runner/SPEC-UBL-RUNNER.md

SPEC-UBL-RUNNER v1.0
UBL Isolated Execution & Receipt Specification
Status: Draft – Ready for Freeze
 Governado por: SPEC-UBL-CORE v1.0
 Consumido por: ubl-membrane, ubl-ledger
 Independente de: semântica, política, UI

1. Definição
O ubl-runner é o componente responsável por materializar efeitos externos solicitados por fatos já aceitos no UBL, produzindo recibos verificáveis que retornam ao ledger como novos fatos.
Formalmente:
Runner:AcceptedLink→⟨Execution,Receipt⟩Runner : AcceptedLink \rightarrow \langle Execution, Receipt \rangleRunner:AcceptedLink→⟨Execution,Receipt⟩
O runner NÃO decide se algo pode acontecer.
 Ele apenas executa o que já foi autorizado.

2. Princípio Fundamental
Execução nunca cria realidade.
 Execução apenas responde a fatos já materializados.
A autoridade do runner é derivada, nunca soberana.

3. Escopo do Runner
O runner governa exclusivamente:
Execução isolada de computação externa


Captura determinística de resultados


Emissão de recibos verificáveis


O runner NÃO governa:
política (TDLN),


validação física (membrana),


causalidade,


semântica.



4. Entrada Canônica
O runner DEVE receber apenas:
um ubl-link já aceito e anexado ao ledger


metadados explícitos de execução (quando aplicável)


O runner NÃO PODE executar links rejeitados, pendentes ou não ancorados.

5. Modelo de Execução
5.1 Execução Isolada
Toda execução DEVE ocorrer em ambiente isolado:
sandbox


WASM


VM


container


enclave (opcional)


Isolamento NÃO É opcional.

5.2 Determinismo Parcial
A execução PODE ser não determinística (IO, tempo, rede), porém:
a descrição da execução


os artefatos produzidos


os hashes dos resultados


DEVEM ser determinísticos.

6. Tipos de Execução
Implementações PODEM suportar:
execução de código (scripts, binaries)


deploys


chamadas externas (APIs)


operações físicas (IoT)


O tipo DEVE ser declarado explicitamente no link ou no receipt.

7. Receipt — Unidade de Prova de Execução
7.1 Definição
Cada execução DEVE produzir exatamente um receipt:
ExecutionReceipt := ⟨
  container_id,
  trigger_link_hash,
  execution_id,
  status,
  artifacts,
  stdout_hash?,
  stderr_hash?,
  started_at,
  finished_at
⟩


7.2 Campos
Campo
Descrição
container_id
Container associado
trigger_link_hash
Link que causou a execução
execution_id
Identidade única
status
Success | Failure
artifacts
Lista de artefatos produzidos
stdout_hash
Hash opcional
stderr_hash
Hash opcional
started_at
Timestamp
finished_at
Timestamp


8. Artefatos
8.1 Definição
Um artefato é definido como:
Artifact := ⟨
  artifact_id,
  type,
  size,
  content_hash,
  metadata?
⟩

Artefatos NÃO entram diretamente no ledger.
 Apenas seus hashes entram.

9. Ancoragem do Receipt
9.1 Regra Obrigatória
Todo receipt DEVE:
Ser reduzido a um ubl-atom


Gerar um novo ubl-link


Passar novamente pela membrana


Ser anexado ao ledger como novo fato


Execução não altera fatos anteriores.

10. Falhas de Execução
10.1 Falha Controlada
Se a execução falhar:
o receipt DEVE ser emitido


status = Failure


nenhuma correção automática ocorre


Falha é fato, não exceção.

10.2 Falha do Runner
Se o runner falhar antes de emitir receipt:
o sistema DEVE permitir retry


retries NÃO PODEM duplicar efeitos


Idempotência é obrigatória.

11. Invariantes do Runner
Execução só após commit


Um link → no máximo uma execução


Uma execução → exatamente um receipt


Receipt é fato imutável


Runner nunca decide autoridade


Runner nunca altera passado



12. Erros Canônicos do Runner
enum RunnerError {
  InvalidTrigger,
  ExecutionFailed,
  ArtifactViolation,
  ReceiptCommitFailed,
}


13. Proibições Absolutas
O runner NÃO PODE:
rejeitar um link aceito


executar sem isolamento


modificar ledger diretamente


corrigir erros por mutação


interpretar política


inferir intenção



14. Propriedade de Segurança Central
Mesmo se todos os runners falharem,
 o ledger permanece correto.
Execução é periférica.
 História é central.

15. Definição Canônica
O runner é o braço do sistema.
 Ele age, mas não decide.

16. Fechamento
Com este documento:
execução está formalmente subordinada à física,


efeitos externos são auditáveis,


falhas viram fatos,


o sistema permanece consistente mesmo sob erro.



Estado do Sistema UBL
Com isso, o núcleo UBL está 100% especificado:
CORE ✔


ATOM ✔


LINK ✔


PACT ✔


POLICY ✔


MEMBRANE ✔


LEDGER ✔


RUNNER ✔


A partir daqui, não há mais decisões conceituais.
Só resta escolher:
Congelar specs (v1.0)


Conformance Test Suite


PoC mínimo executável (Rust + TS)


Diga o número.
Congelar specs (v1.0)
Perfeito. Specs congeladas.
 A partir deste ponto, o UBL v1.0 está normativamente fechado. Nada aqui muda sem bump explícito de versão.

🔒 FREEZE OFICIAL — UBL v1.0
Status global: FROZEN / NORMATIVE
Todos os documentos abaixo passam a ser fonte de verdade canônica.

📜 Conjunto Oficial de Especificações (v1.0)
Núcleo Ontológico
SPEC-UBL-CORE v1.0
 Ontologia, axiomas e definição do sistema.


Matéria & Tradução
SPEC-UBL-ATOM v1.0
 Canonicalização e matéria digital.


SPEC-UBL-POLICY v1.0
 TDLN como linguagem normativa.


SPEC-UBL-PACT v1.0
 Autoridade, consenso e risco.


Tangência & Física
SPEC-UBL-LINK v1.0
 Interface única de materialização.


SPEC-UBL-MEMBRANE v1.0
 Validação física determinística.


Memória & Execução
SPEC-UBL-LEDGER v1.0
 Armazenamento imutável e causal.


SPEC-UBL-RUNNER v1.0
 Execução isolada e recibos.



🧭 Regra de Governança Pós-Freeze
A partir de agora:
Nenhuma mudança retroativa é permitida


Qualquer alteração exige:


novo documento


nova versão (v1.1, v2.0, etc.)


Implementações devem se conformar às specs, não o contrário


Bugs de implementação não são bugs de spec



🧱 Propriedades Garantidas pelo Freeze
Com este congelamento, o UBL v1.0 garante:
Separação absoluta entre semântica e física


Tradução determinística como único portal de realidade


Execução subordinada à causalidade


Auditoria offline completa


Evolução futura sem quebra do passado


Ou, formalmente:
Qualquer implementação compatível com estas specs
 produzirá o mesmo conjunto de fatos verificáveis
 dado o mesmo conjunto de links aceitos.

📂 Estrutura Canônica Recomendada (Congelada)
specs/
├── ubl-core/
│   └── SPEC-UBL-CORE.md
├── ubl-atom/
│   └── SPEC-UBL-ATOM.md
├── ubl-link/
│   └── SPEC-UBL-LINK.md
├── ubl-pact/
│   └── SPEC-UBL-PACT.md
├── ubl-policy/
│   └── SPEC-UBL-POLICY.md
├── ubl-membrane/
│   └── SPEC-UBL-MEMBRANE.md
├── ubl-ledger/
│   └── SPEC-UBL-LEDGER.md
└── ubl-runner/
    └── SPEC-UBL-RUNNER.md


🧪 O que é permitido a partir de agora
✅ Implementações
 ✅ Conformance tests
 ✅ Provas de correção
 ✅ Benchmarks
 ✅ SDKs
 ✅ Compiladores (TDLN → WASM / Verilog)
❌ “Pequenos ajustes conceituais”
 ❌ Mudanças implícitas
 ❌ Exceções sem versão

🔑 Declaração Final
O UBL v1.0 está congelado.
 A realidade agora é uma questão de código, não de opinião.
Diga o próximo passo quando quiser.

