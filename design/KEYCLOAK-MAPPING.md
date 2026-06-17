# Bodhi Keycloak Auth Theme — Porting Reference

Hi-fidelity prototype: one standalone HTML page per Keycloak template, sharing
**`bodhi-auth.css`** + **`bodhi-auth.js`** (and the app's `colors_and_type.css`
+ `bodhi-theme.js`). This is the design source for a Keycloak **login theme**.
It is intentionally structured so the port to real Keycloak templates is
mechanical, not a redesign.

> Not meant to drop in as-is. It's a faithful starting point — keep the markup,
> IDs and class hooks; swap static content for FreeMarker (`${...}`) expressions.

---

## 1. The pages

| File | Keycloak template (`data-kc-page`) | Purpose |
|---|---|---|
| `login.html`            | `login.ftl`                  | Sign in + social providers |
| `register.html`         | `register.ftl`               | Create account |
| `forgot-password.html`  | `login-reset-password.ftl`   | Forgot password |
| `update-password.html`  | `login-update-password.ftl`  | Set new password |
| `verify-email.html`     | `login-verify-email.ftl`     | Verify email (info) |
| `consent.html`          | `login-oauth-grant.ftl`      | OAuth consent / grant |
| `error.html`            | `error.ftl`                  | Generic error / info |

Each page is self-contained and links to the others with normal `<a href>`
(e.g. login → `forgot-password.html`, → `register.html`; the rest → `login.html`),
mirroring how Keycloak links its templates. Start at `login.html`.

**Identifier = email.** There is no separate username field anywhere. The login
and register email inputs keep `id/name="username"` / `email` exactly as Keycloak
expects, but are labelled "Email" and typed `email`.

**No locale switcher** — internationalization is not supported, so it's omitted.

---

## 2. Theme mechanics

- Light/dark is driven by `data-theme="light|dark"` on `<html>`, managed by
  `bodhi-theme.js` (persists to localStorage). The visible **theme toggle**
  (top-right of every page, `#ba-theme`) is a real feature — port it into
  `template.ftl`. In Keycloak you may also seed the initial value from a realm
  attribute, cookie, or `prefers-color-scheme`. All tokens cascade from it.
- All color/spacing/type come from `colors_and_type.css` (the app design system).
  Ship `colors_and_type.css` + `bodhi-auth.css` (+ optional `bodhi-auth.js`) as
  theme resources (`login/resources/…`) and reference via `${url.resourcesPath}`.
- `bodhi-auth.js` is pure UX sugar (password show/hide, strength meter, theme
  toggle) — keep or drop freely.
- Fonts: Inter (UI), loaded from Google Fonts in the prototype — bundle them as
  theme resources for production rather than hot-linking.

---

## 3. Class hook → Keycloak `kcClass` mapping

Bodhi classes use the `ba-` prefix. Attach each to its Keycloak hook in
`theme.properties` (so the stock `${properties.kcXxx}` expressions in the
templates pick up Bodhi styling), **or** override the templates to emit the
`ba-` classes directly. Recommended: map via `theme.properties`.

| Bodhi class | Keycloak property (`theme.properties`) | Notes |
|---|---|---|
| `ba-page`         | `kcBodyClass` (+ `kcLoginClass`)      | full-viewport branded canvas |
| `ba-card`         | `kcFormCardClass`                     | the centered card |
| `ba-brand` / `ba-brand-*` | `kcHeaderClass` / `kcHeaderWrapperClass` | logo + wordmark |
| `ba-alert ba-alert-{error,success,warning,info}` | `kcAlertClass` + type modifiers | feedback messages |
| `ba-form`         | `kcFormClass`                         | `<form>` |
| `ba-group`        | `kcFormGroupClass`                    | label + input wrapper |
| `ba-label`        | `kcLabelClass`                        | field label |
| `ba-input`        | `kcInputClass`                        | text/email/password inputs |
| `ba-input-wrap` / `ba-pw-wrap` | `kcInputWrapperClass`        | input wrapper |
| `ba-field-error`  | `kcInputErrorMessageClass`            | per-field validation text |
| `ba-options`      | `kcFormOptionsClass` / `kcFormSettingClass` | remember-me + forgot row |
| `ba-checkbox`     | `kcCheckboxInputClass`                | |
| `ba-btn ba-btn-primary ba-btn-block` | `kcButtonClass kcButtonPrimaryClass kcButtonBlockClass` | submit |
| `ba-btn-secondary`| `kcButtonClass kcButtonDefaultClass`  | cancel / deny |
| `ba-divider`      | (custom)                              | "Or continue with" |
| `ba-social`       | `kcFormSocialAccountListClass` (`is-grid` → `kcFormSocialAccountListGridClass`) | provider list |
| `ba-social-item`  | `kcFormSocialAccountListButtonClass`  | one provider |
| `ba-foot`         | `kcInfoAreaWrapperClass`              | registration / back links |
| `ba-theme-toggle` | (custom)                              | light/dark toggle in header |

---

## 4. Field / element IDs — keep these exact

These match what Keycloak's templates expect; keep IDs & `name`s so JS,
validation and form submission work unchanged.

**login.ftl** — `#kc-form-login`, `input#username[name=username]` (labelled
"Email", `type=email` — email is the identifier), `input#password[name=password]`,
`input#rememberMe[name=rememberMe]`, `button#kc-login`, social list
`#kc-social-providers`, registration `#kc-registration`.

**register.ftl** — `#kc-register-form`, fields in order `email`, `password`,
`password-confirm`, `firstName`, `lastName`, `termsAccepted`, submit
`#kc-register`. **No `username` field** — Keycloak realm should be configured
with email-as-username so `email` satisfies the username requirement.

**login-reset-password.ftl** — `#kc-reset-password-form`, `input[name=username]`.

**login-update-password.ftl** — `#kc-passwd-update-form`,
`password-new`, `password-confirm`, `logout-sessions`.

**login-oauth-grant.ftl** — `#kc-oauth-grant-form`, list `#kc-oauth-grant-list`,
`button#kc-login[name=accept]`, `button#kc-cancel[name=cancel]`,
app name carries `data-kc-field="client.name"`.

**error.ftl** — message node carries `data-kc-field="message.summary"`.

`data-kc-field` / `data-kc-link` / `data-kc-provider` attributes throughout
mark exactly which FreeMarker expression replaces that static content
(e.g. `data-kc-field="username"` → `<#-- value="${login.username!''}" -->`).
They're annotations — strip them after wiring up the `${...}`.

---

## 5. Dynamic bits to wire on the port

- **Social providers** are hard-coded (Google, Microsoft, Twitter, GitHub) for
  the mock. In `login.ftl` loop `social.providers` and render one
  `ba-social-item` each; the `.is-grid` 2-col layout is right for 4+ providers,
  drop `is-grid` for ≤3 (stacked full-width).
- **Alerts**: render the `ba-alert` block from `message` only when present;
  map `message.type` → `ba-alert-{error|success|warning|info}`.
- **Field errors**: add `is-error` + `aria-invalid` to the input and render
  `ba-field-error` from `messagesPerField.get('email')` etc. (`.ba-field-error`
  markup demonstrates the styling).
- **Password meter** (register / update) is a UX nicety implemented in JS —
  purely client-side, safe to keep or drop.
- **Required markers**: `<span class="ba-req">*</span>` after the label text.
