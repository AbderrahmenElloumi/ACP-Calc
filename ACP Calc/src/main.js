const { invoke } = window.__TAURI__.core;

let InputLines;
let InputColumns;
let MatrixContainer;

let m = [];
let res;

function generateMatrix() {
  const rows = parseInt(InputLines.value);
  const cols = parseInt(InputColumns.value);

  MatrixContainer.innerHTML = ""; // clear previous matrix

  const table = document.createElement("table");
  table.style.borderCollapse = "collapse";

  for (let i = 0; i < rows; i++) {
    const tr = document.createElement("tr");
    for (let j = 0; j < cols; j++) {
      const td = document.createElement("td");
      const input = document.createElement("input");
      input.type = "number";
      input.style.width = "60px";
      input.style.margin = "2px";
      input.name = `cell-${i}-${j}`; // optional: helps identify each cell
      td.appendChild(input);
      tr.appendChild(td);
    }
    table.appendChild(tr);
  }

  MatrixContainer.appendChild(table);
}

async function acp_calc() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  const response = await invoke("acp", { matrix: JSON.stringify(m), threshold: "" });

  const output = JSON.parse(response);
  console.log("ACP Result:", output);
  res.textContent = JSON.stringify(output, null, 2); 
}

window.addEventListener("DOMContentLoaded", () => {
  InputLines = document.querySelector("#lines");
  InputColumns = document.querySelector("#columns");
  MatrixContainer = document.querySelector("#matrix");
  res = document.querySelector("#result");

  document.querySelector("#size-form").addEventListener("submit", (e) => {
    e.preventDefault();
    generateMatrix();
  });

  document.querySelector("#matrix-form").addEventListener("submit", (e) => {
    e.preventDefault();

    for (let i = 0; i < parseInt(InputLines.value); i++) {
      m[i] = [];
      for(let j = 0; j < parseInt(InputColumns.value); j++) {
        m[i][j] = parseFloat(MatrixContainer.querySelector(`input[name="cell-${i}-${j}"]`).value);
        console.log(`m[${i}][${j}] = ${m[i][j]}`);
      }
    }

    console.log("Matrix m:", m);
    acp_calc();
  });
});