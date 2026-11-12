## {{ version.tag }} - {{ date | truncate(length=10, end="") }}{%- set wanted = ["Features", "Bug Fixes", "Performance Improvements", "Refactoring"] -%}
{%- set_global breaking = false -%}
{%- for commit in commits -%}
{%- if commit.breaking_change -%}
{%- set_global breaking = true -%}
{%- endif -%}
{%- endfor -%}
{%- if breaking %}

#### Breaking Changes
{%- for commit in commits %}
{%- if commit.breaking_change %}

- ({{ commit.id | truncate(length=7, end="") }}) {{ commit.summary }}{% for footer in commit.footers %}{% if footer.footer.token == "BREAKING CHANGE" %}: {{ footer.footer.content }}{% endif %}{% endfor %} - {{ commit.signature }}
{%- endif %}
{%- endfor %}
{%- endif -%}
{%- for title in wanted -%}
{%- set group = commits | filter(attribute="type", value=title) -%}
{%- if group | length > 0 %}

#### {{ title }}
{%- for commit in group %}

- ({{ commit.id | truncate(length=7, end="") }}) {{ commit.summary }} - {{ commit.signature }}
{%- endfor -%}
{%- endif -%}
{%- endfor %}
