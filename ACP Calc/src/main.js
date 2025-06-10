const { invoke } = window.__TAURI__.core;

let Threshold;
let ThreshRes;

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
      input.step = "any"; // allows decimal input
      input.value = "0"; // default value
      input.style.width = "60px";
      input.style.margin = "2px";
      input.name = `cell-${i}-${j}`; // helps identify each cell
      td.appendChild(input);
      tr.appendChild(td);
    }
    table.appendChild(tr);
  }

  MatrixContainer.appendChild(table);
}

function renderResult(resultData) {
  res.innerHTML = ""; // Clear old result

  Object.entries(resultData).forEach(([title, content]) => {
    const section = document.createElement("div");
    section.style.marginBottom = "20px";

    const heading = document.createElement("h3");
    heading.textContent = title;
    section.appendChild(heading);

    // If it's a matrix/table section
    if (content.columns && content.data && content.index) {
      // Create toggle button
      const toggleButton = document.createElement("button");
      toggleButton.textContent = "Show Table";
      section.appendChild(toggleButton);

      const table = document.createElement("table");
      table.style.borderCollapse = "collapse";
      table.style.border = "1px solid black";
      table.style.margin = "auto";
      table.style.display = "none"; // Initially hidden

      const thead = document.createElement("thead");
      const headerRow = document.createElement("tr");

      const emptyHeader = document.createElement("th");
      headerRow.appendChild(emptyHeader);

      content.columns.forEach((col) => {
        const th = document.createElement("th");
        th.textContent = col;
        headerRow.appendChild(th);
      });

      thead.appendChild(headerRow);
      table.appendChild(thead);

      const tbody = document.createElement("tbody");

      content.data.forEach((row, i) => {
        const tr = document.createElement("tr");
        const rowHeader = document.createElement("td");
        rowHeader.textContent = content.index[i];
        tr.appendChild(rowHeader);

        row.forEach((cell) => {
          const td = document.createElement("td");
          td.textContent = cell;
          td.style.border = "1px solid black";
          td.style.padding = "4px";
          tr.appendChild(td);
        });

        tbody.appendChild(tr);
      });

      table.appendChild(tbody);
      section.appendChild(table);

      // Add click toggle logic
      toggleButton.addEventListener("click", () => {
        if (table.style.display === "none") {
          table.style.display = "table";
          toggleButton.textContent = "Hide Table";
        } else {
          table.style.display = "none";
          toggleButton.textContent = "Show Table";
        }
      });
    }
    // If it's a list of messages
    else if (Array.isArray(content)) {
      // Create toggle button
      const toggleButton = document.createElement("button");
      toggleButton.textContent = "Show Actions";
      section.appendChild(toggleButton);

      const ul = document.createElement("ul");
      ul.style.listStyleType = "none"; // Remove default list styling
      ul.style.margin = "auto";
      ul.style.display = "none"; // Initially hidden
      content.forEach((line) => {
        const li = document.createElement("li");
        li.textContent = line;
        ul.appendChild(li);
      });
      section.appendChild(ul);

      // Add click toggle logic
      toggleButton.addEventListener("click", () => {
        if (ul.style.display === "none") {
          ul.style.display = "table";
          toggleButton.textContent = "Hide Actions";
        } else {
          ul.style.display = "none";
          toggleButton.textContent = "Show Actions";
        }
      });   
    }

    res.appendChild(section);
  });
}

async function acp_calc() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  try {
    const response = await invoke("acp", { matrix: JSON.stringify(m), threshold: Threshold.value });

    console.log("ACP Result:", response);
    // res.textContent = JSON.stringify(response, null, 2); 
    renderResult(response);
  } catch (error) {
    console.error("ACP Error", error);
    res.textContent = `Error:\n ${error}`;
  }
}

window.addEventListener("DOMContentLoaded", () => {
  Threshold = document.querySelector("#threshold");
  ThreshRes = document.querySelector("#thresh-res");
  InputLines = document.querySelector("#lines");
  InputColumns = document.querySelector("#columns");
  MatrixContainer = document.querySelector("#matrix");
  res = document.querySelector("#result");

  document.querySelector("#threshold-form").addEventListener("submit", (e) => {
    e.preventDefault();
    const thresholdValue = parseFloat(Threshold.value);
    if (isNaN(thresholdValue) || thresholdValue <= 0) {
      ThreshRes.textContent = "Please enter a valid positive number for the threshold.";
      return;
    }
    Threshold.value = thresholdValue.toFixed(5); // format to 5 decimal places
    ThreshRes.textContent = `Threshold set to ${Threshold.value}`;
  });

  document.querySelector("#size-form").addEventListener("submit", (e) => {
    e.preventDefault();
    generateMatrix();
  });

  document.querySelector("#matrix-form").addEventListener("submit", (e) => {
    e.preventDefault();

    for (let i = 0; i < parseInt(InputLines.value); i++) {
      m[i] = [];
      for(let j = 0; j < parseInt(InputColumns.value); j++) {
        const val = MatrixContainer.querySelector(`input[name="cell-${i}-${j}"]`).value;
        if (val === "") {
          res.textContent = "Please fill all matrix cells.";
          return;
        }
        m[i][j] = parseFloat(val);
        console.log(`m[${i}][${j}] = ${m[i][j]}`);
      }
    }

    console.log("Matrix m:", m);
    acp_calc();
  });
});