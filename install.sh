#!/usr/bin/env bash

# Sai imediatamente se houver algum erro
set -e

REPO="DuckCosmos/waylaw"
BIN_NAME="waylaw"
INSTALL_DIR="$HOME/.local/bin"

echo "=== Instalador do Waylaw ==="

if command -v pacman &> /dev/null; then
    echo "Verificando dependências (gtk4 e gtk4-layer-shell)..."
    if ! pacman -Qi gtk4 &> /dev/null || ! pacman -Qi gtk4-layer-shell &> /dev/null; then
        echo "Aviso: Você precisa do gtk4 e gtk4-layer-shell instalados."
        echo "Você pode instalá-los rodando: sudo pacman -S gtk4 gtk4-layer-shell"
        echo "Continuando a instalação do binário..."
    fi
fi
echo "Buscando a versão mais recente no GitHub..."
LATEST_URL=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep "browser_download_url.*x86_64.tar.gz" | cut -d '"' -f 4)

if [ -z "$LATEST_URL" ]; then
    echo "Erro: Não foi possível encontrar o arquivo .tar.gz na última Release do GitHub."
    echo "Verifique se o repositório $REPO realmente tem uma Release pública."
    exit 1
fi

# 3. Baixar o arquivo
echo "Baixando: $LATEST_URL"
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"
curl -sL -O "$LATEST_URL"

# 4. Extract
echo "Extraindo arquivos..."
tar -xzf *.tar.gz

# 5. Install
echo "Instalando em $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
mv "$BIN_NAME" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$BIN_NAME"

# 6. Clear
cd - > /dev/null
rm -rf "$TMP_DIR"

echo "=== Instalação concluída com sucesso! ==="
echo "Certifique-se de que $INSTALL_DIR está no seu PATH."
echo "Para abrir o launcher, execute: $BIN_NAME"
echo "Adicione ao seu hyprland.conf: bind = SUPER, SPACE, exec, $BIN_NAME"
