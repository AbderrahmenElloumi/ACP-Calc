
#TODO: Make GUI for input
#TODO: Think up a way to handle input dynamically
#TODO: Configure the existing code to handle automation better 
#TODO: Look for // Optimisation


import numpy as np
from pandas import DataFrame
from tabulate import tabulate
import sys
import json
from ast import literal_eval
from typing import Union


def cov(X):
    #Les moyennes, on travaille avec des données centrées, donc, on n'a pas besoin de soustraire les moyennes
    return np.dot(X.T, X) / (X.shape[0]) 


def restrict(Y, v, rapport,frmt):
    global p
    l = len(v)
    a = 0
    d = []
    s = []
    # np.sum(v[:l]) est égale à p
    for j in range(l):
        a = np.sum(v[:j])/np.sum(v[:l])
        if a > rapport :
            s.append(f"--> Suppression de la colonne X{j+1} | Rapport: {a}")
            p = p - 1
            d.append(frmt+str(j+1))
    return Y.drop(d, axis=1), s

    
def create_dynamic_df(data, rows, cols, srow, scol):
    col_names = [scol + str(i+1) for i in range(cols)]
    row_names = [srow + str(i+1) for i in range(rows)]
    return DataFrame(data, columns=col_names, index=row_names)


def matrix_from_js(data: Union[str, list]) -> np.ndarray:
    
    if isinstance(data, str):
        try:
            # Try to parse as JSON string
            parsed = json.loads(data)
        except json.JSONDecodeError:
            # Attempt to eval raw string version (unsafe for untrusted input)
            try:
                parsed = literal_eval(data, {"__builtins__": {}})
            except Exception as e:
                raise ValueError(f"Failed to parse matrix from string: {e}")
    elif isinstance(data, list):
        parsed = data
    else:
        raise ValueError("Input must be a JSON string, raw matrix string, or list of lists.")

    # Ensure it's a matrix (2D list of floats)
    try:
        array = np.array(parsed, dtype=float)
        if array.ndim != 2:
            raise ValueError("Input is not a 2D matrix.")
        return array
    except Exception as e:
        raise ValueError(f"Failed to convert to NumPy array: {e}")


def json_output(data: dict) -> str:
    for key, value in data.items():
        if isinstance(value, DataFrame):
            data[key] = value.to_dict(orient='split')
        elif isinstance(value, list):
            data[key] = [str(item) for item in value]
    with open("output.json", "w") as f:
        json.dump(data, f, indent=4)
        
#!Principal:

#*Gérération de la matrice de données

if len(sys.argv) != 3:
    raise ValueError("Usage: python script.py <matrix_string> <threshold>")

M = matrix_from_js(sys.argv[1])
n = M.shape[0]
if n < 2:
    raise ValueError("The matrix must have at least 2 rows (individuals).")
p = M.shape[1]
l = {}
tol = float(sys.argv[2])


dfM = create_dynamic_df(M, n, p, "Individu ", "X")
l["Matrice de départ"] = dfM


#*Centrage et réduction de la matrice de données
Moyenne = [np.mean(M, axis=0).round(3)]
Ecart_type = [np.std(M, axis=0).round(3)]
l["Vecteurs Moyennes"] = create_dynamic_df(Moyenne, 1, p, "Moyenne ", "X")
l["Vecteurs Ecart-types"] = create_dynamic_df(Ecart_type, 1, p, "Ecart-Type ", "X")

li = np.where(Ecart_type[0] == 0)
if np.any(Ecart_type[0] == 0):
    zero_std_columns = [f"X{col + 1}" for col in li[0]]
    raise ValueError(f"At least one column has zero standard deviation. Cannot perform ACP. Columns: {', '.join(zero_std_columns)}")

Z = (M - Moyenne) / Ecart_type
Z = np.round(Z, 2)
dfZ = create_dynamic_df(Z, n, p, "Individu ", "X")
l["Matrice centrée Réduite"] = dfZ


#*Matrice de corrélation
R = cov(Z) #Corrélation = Covariance après centrage et réduction
R = np.round(R, 2)
dfR = create_dynamic_df(R, p, p, "X", "X")
l["Matrice de Corrélation"] = dfR


#*Calcul des valeurs propres et vecteurs propres
valeurs_propres, vecteurs_propres = np.linalg.eig(R)
ordre = np.argsort(valeurs_propres)[::-1]
valeurs_propres = valeurs_propres[ordre]
vecteurs_propres = vecteurs_propres[:, ordre]


dfV = create_dynamic_df(valeurs_propres, p, 1, "Valeur propre ", "")
l["Valeurs propres"] = dfV.round(2)



#*Matrice Q
l["Normes des Vecteurs propres"] = create_dynamic_df(np.linalg.norm(vecteurs_propres, axis=0).round(3), p, 1, "Norme ", "V.P. ")
Q = vecteurs_propres
Q = np.round(Q, 2)
dfQ = create_dynamic_df(Q, p, p, "", "V.P. ")
l["Matrice Q"] = dfQ


#*Nouvelle Matrice de données
Y = np.dot(Z, Q)
Y = np.round(Y, 3)
dfY = create_dynamic_df(Y, n, p, "Individu ", "X")
l["Nouvelle matrice de données"]  = dfY


#*Matrice après restriction
dfY, l["Suppression"] = restrict(dfY, valeurs_propres, tol, "X")
l["Matrice après restriction"] = dfY

json_output(l)


