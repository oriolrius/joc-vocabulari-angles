# The game as a static image: index.html + vocabulari/ served by nginx (unprivileged, read-only).
# CI: .github/workflows/image.yml (on each tag v*). Production: k5 stack k5-stack-angles.
FROM nginxinc/nginx-unprivileged:1.31.7-alpine3.24@sha256:b5047b9b76845eac124358373c6852377c52e4b05834a47647e955dd91cb8f1e

ARG VERSION=dev
ARG REVISION=dev
LABEL org.opencontainers.image.title="joc-vocabulari-angles" \
      org.opencontainers.image.description="Joc de Vocabulari Anglès<>Català" \
      org.opencontainers.image.source="https://github.com/oriolrius/joc-vocabulari-angles" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${REVISION}"

USER root
COPY deploy/nginx.conf /etc/nginx/conf.d/default.conf
COPY index.html /usr/share/nginx/html/
COPY vocabulari/ /usr/share/nginx/html/vocabulari/
# Files checked out with mode 0600 (umask 077) would give 403: make every file world-readable.
RUN sed -i -e "s/__VERSION__/${VERSION}/g" -e "s/__REVISION__/${REVISION}/g" /etc/nginx/conf.d/default.conf \
 && chmod 0644 /etc/nginx/conf.d/default.conf \
 && chmod -R a+rX /usr/share/nginx/html \
 && nginx -t
USER 101

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --retries=3 CMD wget -qO /dev/null http://127.0.0.1:8080/healthz || exit 1
# No entrypoint scripts: they try to edit the config, and the file system is read-only.
ENTRYPOINT []
CMD ["nginx", "-g", "daemon off;"]
