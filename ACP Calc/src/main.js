const { invoke } = window.__TAURI__.core;

let Threshold;
let ThreshRes;

let ResWarning;

let InputLines;
let InputColumns;
let MatrixContainer;

let data = null;
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
      input.style.width = "80px";
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
  data = resultData;
  res.innerHTML = ""; // Clear old result

  const keyOrder = ["Matrice de d\u00e9part", 
                    "Vecteurs Moyennes",
                    "Vecteurs Ecart-types", 
                    "Matrice centr\u00e9e R\u00e9duite", 
                    "Matrice de Corr\u00e9lation", 
                    "Valeurs propres", "Normes des Vecteurs propres", 
                    "Matrice Q", 
                    "Nouvelle matrice de donn\u00e9es",
                    "Suppression", 
                    "Matrice apr\u00e8s restriction"];
             
  keyOrder.forEach((title) => {
    if (!(title in resultData)) console.log(`Missing key: ${title}, index: ${keyOrder.indexOf(title)}`);

    const content = resultData[title];
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

function ButtonType(resElement, buttonLabel, typeMessage) {
  const NewButton = document.createElement("button");
  NewButton.textContent = buttonLabel;

  NewButton.addEventListener("click", (event) => {
    event.stopPropagation(); // Prevent triggering the parent click
    resElement.innerHTML = 
      `Please make sure your <b>${buttonLabel}</b> file is formatted correctly.<br>${typeMessage.replace(/\n/g, '<br>')}`;
    resElement.dataset.state = "expanded"; // Mark as expanded
  });

  resElement.appendChild(document.createElement("br")); // Line break before button
  resElement.appendChild(NewButton);
}

function showWarning(message) {
  const ResWarning = document.querySelector("#warning");

  if (ResWarning.dataset.state !== "expanded") {
    // Clear and prepare new content
    ResWarning.innerHTML = message.replace(/\n/g, "<br>");

    ButtonType(ResWarning, ".csv", "CSV files should have a header row and values separated by commas.\nFirst row headers are not supported!");
    ButtonType(ResWarning, ".json", "JSON files should strictly be an array of number arrays!");
    ButtonType(ResWarning, ".xml", "XML files should have a structured format with the proper tags so that it can be converted to a matrix format.\nPlease use <matrix> as the root element and <row> for each row\nFirst row headers are not supported!");
    ButtonType(ResWarning, ".txt", "Text files should have values separated by spaces or tabs.\nFirst row headers are not supported!");
    ButtonType(ResWarning, ".ods", "Excel files should have numeric values in cells.\nFirst row headers are not supported!\nEnsure forcing full recalculation of the sheet by clicking Ctrl+Alt+Shift+F9 in your Excel editor.");
    ButtonType(ResWarning, ".xlsm", "Excel files should have numeric values in cells.\nFirst row headers are not supported!\nEnsure forcing full recalculation of the sheet by clicking Ctrl+Alt+Shift+F9 in your Excel editor.");
    ButtonType(ResWarning, ".xlsx", "Excel files should have numeric values in cells.\nFirst row headers are not supported!\nEnsure forcing full recalculation of the sheet by clicking Ctrl+Alt+Shift+F9 in your Excel editor.");
    ButtonType(ResWarning, ".xls", "Excel files should have numeric values in cells.\nFirst row headers are not supported!\nEnsure forcing full recalculation of the sheet by clicking Ctrl+Alt+Shift+F9 in your Excel editor.");
    
    ResWarning.dataset.state = "expanded";
  } else {
    // Reset to original
    ResWarning.textContent = "Warning Notes";
    ResWarning.dataset.state = "collapsed"; // Reset state
  }
}

async function loadMatrixWithDialog() {
  try {
    const matrix = await invoke("load_matrix_with_dialog");
    m = JSON.parse(matrix);
    
    // Update UI with loaded matrix
    InputLines.value = m.length;
    InputColumns.value = m[0].length;
    generateMatrix();
    
    // Fill in the values
    m.forEach((row, i) => {
      row.forEach((val, j) => {
        MatrixContainer.querySelector(`input[name="cell-${i}-${j}"]`).value = val;
      });
    });
    
    res.textContent = "Matrix loaded successfully!";
  } catch (error) {
    console.error("File loading error:", error);
    res.textContent = `Error loading file: ${error}`;
  }
}

async function acp_calc(threshold = 0.925) {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  try {
    const response = await invoke("acp", { matrix: JSON.stringify(m), threshold: threshold });

    console.log("ACP Result:", response);
    // res.textContent = JSON.stringify(response, null, 2); 
    renderResult(response);
  } catch (error) {
    console.error("ACP Error", error);
    res.textContent = `Error:\n ${error}`;
  }
}

async function exportMatrix(format = "csv", whichMatrix = "Matrice apr\u00e8s restriction", fileName = "DataAfterACP") {
  try {
    console.log("Exporting with args:", {
      acpresult: JSON.stringify(data),
      format: format,
      which_matrix: whichMatrix,
      file_name: fileName
    });
    
    const response = await invoke("export_matrix", {
      acpresult: JSON.stringify(data),
      format: format,
      whichmatrix: whichMatrix,
      fne: fileName
    });

    console.log("Export Result:", response);
    const exportReport = document.createElement("div");
    exportReport.style.marginBottom = "10px";
    exportReport.textContent = `Exported successfully as ${format.toUpperCase()}`;

    document.querySelector("#export-options").appendChild(exportReport);
    setTimeout(() => exportReport.remove(), 3000);

  } catch (error) {
    console.error("Export Error", error);
    const exportReport = document.createElement("div");
    exportReport.style.marginBottom = "10px";
    exportReport.textContent = `Export failed as ${error}`;
    
    document.querySelector("#export-options").appendChild(exportReport);
    setTimeout(() => exportReport.remove(), 3000);
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
      ThreshRes.textContent = "Please enter a valid positive number for the threshold.<br>0 < threshold < 1.";
      ThreshRes.innerHTML = ThreshRes.textContent; // Convert to HTML to allow line breaks
      return;
    }
    Threshold.value = thresholdValue.toFixed(5); // format to 5 decimal places
    ThreshRes.textContent = `Threshold set to ${Threshold.value}`;
  });
  
  document.querySelector("#warning").addEventListener("click", (e) => {
    e.preventDefault();
    showWarning("Supported Files Types\n");
  });


  // Updated to use dialog instead of file input
  document.querySelector("#load-file").addEventListener("click", (e) => {
    e.preventDefault();
    loadMatrixWithDialog();
  });

  document.querySelector("#size-form").addEventListener("submit", (e) => {
    e.preventDefault();
    document.querySelector("#export-options").style.display = "none";
    generateMatrix();
  });

  document.querySelector("#matrix-form").addEventListener("submit", (e) => {
    e.preventDefault();

    for (let i = 0; i < parseInt(InputLines.value); i++) {
      m[i] = [];
      for (let j = 0; j < parseInt(InputColumns.value); j++) {
        const val = MatrixContainer.querySelector(`input[name="cell-${i}-${j}"]`).value;
        if (val === "") {
          res.textContent = "Please fill all matrix cells.";
          document.querySelector("#export-options").style.display = "none"; // hide if error
          return;
        }
        m[i][j] = parseFloat(val);
        console.log(`m[${i}][${j}] = ${m[i][j]}`);
      }
    }
    // Show the export options after validation
    document.querySelector("#export-options").style.display = "block";
    
    console.log("Matrix m:", m);
    acp_calc(Threshold.value);
  });

  document.querySelector("#export-options").addEventListener("submit", (e) => {
    e.preventDefault();
    const format = document.querySelector("#export-format").value;
    const whichMatrix = document.querySelector("#exported-matrix").value;
    const fileName = document.querySelector("#export-filename").value;
    console.log(`Exporting as ${format} for ${whichMatrix} with filename ${fileName}`);
    exportMatrix(format, whichMatrix, fileName);
  });
});