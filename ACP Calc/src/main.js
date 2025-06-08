const { invoke } = window.__TAURI__.core;

let InputLines;
let InputColumns;
let MatrixContainer;


window.addEventListener("DOMContentLoaded", () => {
  InputLines = document.querySelector("#lines");
  InputColumns = document.querySelector("#columns");
  MatrixContainer = document.querySelector("#matrix");

  document.querySelector("#size-form").addEventListener("submit", (e) => {
    e.preventDefault();
    generateMatrix();
  });
});

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