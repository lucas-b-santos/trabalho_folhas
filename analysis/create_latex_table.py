import pandas as pd

# Caminhos dos arquivos enviados
file = "estatisticas_descritivas.csv"

# Função para converter DataFrame para tabela LaTeX formatada
def df_to_latex(df: pd.DataFrame, caption, label):
    return df.to_latex(
        index=False,
        caption=caption,
        label=label,
        escape=False,
        column_format=" ".join("c" * len(df.columns)),
        # float_format="%.2f",
        bold_rows=False,
        decimal=","
    )
    
# Conversão
latex = df_to_latex(pd.read_csv(file), "Estatísticas Descritivas", "tab:estatisticas")
print(latex)