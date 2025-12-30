# RUNBOOK — Aplicação dos Patches

## Estrutura
- P0/
  - pwa_p0.patch
  - pwa_ui_polish_v1.diff
  - manifest.orange.json
  - manifest.blue.json
  - ubl_jpg_icons_pack.zip (já extraído em assets/icons/jpg/)
- P1/
  - pwa_p1.patch
  - role_theme_patch.diff
  - branding_pack_v2.diff
- P2/
  - maskable_icon_512.png
  - manifest.shortcuts.json
  - manifest.share_target.json
- assets/
  - icons/jpg/*  (ícones prontos em JPG — orange e blue)
  - (opcional) ubl_messenger_icons_orange.zip / ubl_messenger_icons_blue.zip
  - (opcional) ubl_hex_orange.zip / ubl_hex_blue.zip

## Passo-a-passo (git)
1) **Criar branch**
```bash
git checkout -b chore/pwa-bundle
```

2) **Aplicar P0**
```bash
git apply P0/pwa_p0.patch || git apply P0/pwa_ui_polish_v1.diff
```
> Se o patch pedir paths diferentes, rode `git status` e ajuste `--directory` ou `--reject`. 

3) **Manifests por role (temporário)**
- Para laranja (default): use `P0/manifest.orange.json` como `public/manifest.json`.
- Para azul (admin): use `P0/manifest.blue.json` como `public/manifest.json`.
> Em P1, isso passa a trocar automaticamente via claim de login.

4) **Ícones**
- Copie `assets/icons/jpg/*` para o diretório de assets do app (`public/icons/` ou conforme sua estrutura).

5) **Aplicar P1**
```bash
git apply P1/pwa_p1.patch || true
git apply P1/role_theme_patch.diff || true
git apply P1/branding_pack_v2.diff || true
```
> P1 faz: claim → tema, presence badges, prefetch, splash iOS.

6) **P2 (opcional)**
- Coloque `P2/maskable_icon_512.png` em `public/icons/` e referencie no `manifest.json`:
```json
"icons": [
  { "src": "/icons/maskable_icon_512.png", "sizes": "512x512", "type": "image/png", "purpose": "maskable any" }
]
```
- Mescle os campos de `manifest.shortcuts.json` e `manifest.share_target.json` no `manifest.json` principal.

## Smoke Test (iPhone & Mac)
- Instalar PWA → abrir pelo ícone.
- Composer cresce até 6 linhas, sem pular.
- Modo avião → enviar → fechar → voltar online → mensagem sobe (idempotente).
- Job card “Approve” clicado várias vezes → apenas 1 transição no timeline.
- Logar admin → tema azul; usuário normal → laranja.
- SSE reconecta em < 2s após suspender/retomar.

Boa navegação!