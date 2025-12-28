# SPEC-UBL-POLICY v1.0

**TDLN — Deterministic Translation of Language to Notation**

**Status:** FROZEN / NORMATIVE  
**Date:** 2025-12-25  
**Governed by:** SPEC-UBL-CORE v1.0  
**Consumed by:** ubl-cortex, ubl-agent, ubl-link, ubl-pact

---

## 1. Definição

`ubl-policy` define o TDLN como a linguagem normativa que governa:
- como intenções locais podem ser traduzidas em fatos verificáveis,
- sob quais condições um `ubl-atom` é considerado válido,
- quais classes físicas (IntentClass) uma tradução pode produzir,
- quais pactos são exigidos antes da materialização.

**TDLN não executa ações.**  
**TDLN não descreve semântica.**  
**TDLN governa traduções possíveis.**

## 2. Natureza do TDLN

TDLN é:
- Determinístico
- Totalmente verificável
- Semanticamente cego
- Compilável

TDLN NÃO É:
- uma linguagem de workflow,
- uma linguagem de negócio,
- uma linguagem de execução,
- uma linguagem interpretativa.

## 3. Papel Sistêmico

TDLN existe exatamente entre:
- **Linguagem Local (L)**
- **ubl-atom (A)**

Formalmente:

```
TDLN : (Intent, Context) → { AllowedTranslation }
```

Ou seja:  
Dado um estado local, TDLN responde **"isso pode virar um átomo?"**

## 4. Unidade Fundamental: Policy Rule

### 4.1 Definição

Uma Policy Rule define condições de tradução, não efeitos.

```
Rule := ⟨
  rule_id,
  applies_to,
  intent_class,
  constraints,
  required_pact
⟩
```

### 4.2 Campos

| Campo | Descrição |
|-------|-----------|
| `rule_id` | Identidade da regra |
| `applies_to` | Domínio local (container, namespace, tipo) |
| `intent_class` | Classe física resultante permitida |
| `constraints` | Restrições determinísticas |
| `required_pact` | Pacto exigido (opcional) |

## 5. Constraints (Restrições)

### 5.1 Definição

Constraints são predicados determinísticos avaliados antes da tradução.

**Exemplos permitidos:**
- limites numéricos,
- estado lógico (ativo/inativo),
- flags de versão,
- janelas temporais explícitas.

**Exemplos proibidos:**
- heurísticas,
- inferência probabilística,
- acesso a LLM,
- leitura de linguagem natural.

## 6. Resultado da Avaliação

A avaliação de TDLN NUNCA produz efeitos.

Ela produz apenas:

```rust
TranslationDecision :=
  Allow(
    intent_class,
    constraints_snapshot,
    required_pact
  )
  | Deny(reason)
```

## 7. Relação com ubl-atom

TDLN NÃO define o conteúdo do átomo.

TDLN governa apenas:
- se um átomo pode ser gerado,
- qual classe física ele terá,
- qual pacto será exigido.

O formato e o conteúdo do átomo são exclusivamente responsabilidade da linguagem local.

## 8. Relação com ubl-link

TDLN NÃO cria o `ubl-link`.

TDLN produz os parâmetros normativos que o `ubl-link` DEVE respeitar:
- `intent_class`
- limites de `physics_delta`
- exigência de pacto

Qualquer divergência entre:
- decisão TDLN
- conteúdo do `ubl-link`

resulta em rejeição pela membrana.

## 9. Compilação do TDLN

### 9.1 Alvos de Compilação

Uma política TDLN DEVE ser compilável para:
- WASM (execução segura)
- bytecode verificável
- representação lógica (SMT / constraints)

Implementações PODEM gerar:
- CUDA
- Verilog
- eBPF

### 9.2 Propriedade Obrigatória

A política compilada DEVE produzir o mesmo resultado que a política fonte.

## 10. Versionamento e Evolução

### 10.1 Regra de Ouro

Políticas NUNCA são alteradas retroativamente.

Cada commit referencia explicitamente:
- versão da política aplicada,
- hash da política compilada.

## 11. Invariantes do TDLN

- Tradução precede materialização.
- Política não executa efeitos.
- Política não interpreta semântica.
- Política não observa execução.
- Política é determinística.
- Política é auditável offline.

## 12. Proibições Explícitas

TDLN NÃO PODE:
- acessar ledger
- modificar estado
- gerar side effects
- chamar agentes
- depender de tempo implícito
- depender de estado externo não declarado

## 13. Definição Canônica

**TDLN é a lei que governa quais significados podem se tornar fatos no UBL.**

Ou, de forma equivalente:

**UBL não executa intenções.**  
**UBL executa traduções autorizadas.**

---

*Este documento está congelado como parte do UBL v1.0 FROZEN / NORMATIVE.*  
*Nenhuma alteração é permitida sem bump de versão explícito.*
