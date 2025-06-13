
#TODO: Make GUI for input
#TODO: Think up a way to handle input dynamically
#TODO: Configure the existing code to handle automation better 
#TODO: Look for // Optimisation


import numpy as np
from pandas import DataFrame
from tabulate import tabulate


def cov(X):
    #Les moyennes, on travaille avec des données centrées, donc, on n'a pas besoin de soustraire les moyennes
    return np.dot(X.T, X) / (X.shape[0]) 


def restrict(Y, v, rapport,frmt):
    global p
    l = len(v)
    a = 0
    d = []
    # np.sum(v[:l]) est égale à p
    for j in range(l):
        a = np.sum(v[:j])/np.sum(v[:l])
        if a > rapport :
            print(f"--> Suppression de la colonne X{j+1} | Rapport: {a}")
            p = p - 1
            d.append(frmt+str(j+1))
    return Y.drop(d, axis=1)

    
def create_dynamic_df(data, rows, cols, srow, scol):
    col_names = [scol + str(i+1) for i in range(cols)]
    row_names = [srow + str(i+1) for i in range(rows)]
    return DataFrame(data, columns=col_names, index=row_names)


#!Principal:

# n = int(input("Donner le Nbre d'individus: "))
# p = int(input("Donner le Nbre de variables: "))

#*Gérération de la matrice de données
# M = np.random.randint(1000, size=(n, p))
# M = np.round(M, 3) * 1000
M = np.random.randint(100, size=(3, 5))
n = M.shape[0]  # Nombre d'individus
p = M.shape[1]  # Nombre de variables

# M = np.array([[1,2,3],
#               [4,5,6],
#               [7,8,9]])

dfM = create_dynamic_df(M, n, p, "Individu ", "X")
print("\n----------Matrice de départ----------")
print(tabulate(dfM, headers='keys', tablefmt='fancy_grid'))


#*Centrage et réduction de la matrice de données
Moyenne = [np.mean(M, axis=0)]
Ecart_type = [np.std(M, axis=0)]
print(tabulate(create_dynamic_df(Moyenne, 1, p, "Moyenne ", "X"), headers='keys', tablefmt='fancy_grid'))
print(tabulate(create_dynamic_df(Ecart_type, 1, p, "Ecart-Type ", "X"), headers='keys', tablefmt='fancy_grid'))
Z = (M - Moyenne) / Ecart_type
Z = np.round(Z, 2)
dfZ = create_dynamic_df(Z, n, p, "Individu ", "X")
print("\n-----------Matrice centrée Réduite----------")
print(tabulate(dfZ, headers='keys', tablefmt='fancy_grid'))


#*Matrice de corrélation
R = cov(Z) #Corrélation = Covariance après centrage et réduction
R = np.round(R, 2)
dfR = create_dynamic_df(R, p, p, "X", "X")
print("\n-----------Matrice de Corrélation----------")
print(tabulate(dfR, headers='keys', tablefmt='fancy_grid'))


#*Calcul des valeurs propres et vecteurs propres
valeurs_propres, vecteurs_propres = np.linalg.eig(R)
ordre = np.argsort(valeurs_propres)[::-1]
valeurs_propres = valeurs_propres[ordre]
vecteurs_propres = vecteurs_propres[:, ordre]


dfV = create_dynamic_df(valeurs_propres, p, 1, "Valeur propre ", "")
print("\n-----------Valeurs propres----------")
print(tabulate(dfV.round(2), headers='keys', tablefmt='fancy_grid'))



#*Matrice Q
print("\n-----------Normes des Vecteurs propres----------")
print(np.linalg.norm(vecteurs_propres, axis=0))
Q = vecteurs_propres
Q = np.round(Q, 2)
dfQ = create_dynamic_df(Q, p, p, "", "V.P. ")
print("\n-----------Matrice Q----------")
print(tabulate(dfQ, headers='keys', tablefmt='fancy_grid'))


#*Nouvelle Matrice de données
Y = np.dot(Z, Q)
Y = np.round(Y, 3)
dfY = create_dynamic_df(Y, n, p, "Individu ", "X")
print("\n-----------Nouvelle matrice de données----------") 
print(tabulate(dfY, headers='keys', tablefmt='fancy_grid'))


#*Matrice après restriction
dfY = restrict(dfY, valeurs_propres, 0.925, "X")
print("\n-----------Matrice après restriction----------")
print(tabulate(dfY, headers='keys', tablefmt='fancy_grid'))



