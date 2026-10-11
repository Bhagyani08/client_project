import sys
import odxtools
from odxtools.database import Database
from odxtools.diaglayercontainer import DiagLayerContainer
from odxtools.odxlink import OdxLinkId, OdxDocFragment
from odxtools.exceptions import OdxWarning
from metadata import add_admin_data, add_company_datas, add_additional_audiences
from shared import add_functional_classes, add_state_charts
from shared_units import add_common_units, add_common_datatypes
from sessions import add_common_diag_comms
from comparams import generate_comparam_refs
from odxtools.basevariant import BaseVariantRaw
from odxtools.diaglayer import DiagLayerType
from odxtools.parentref import ParentRef
from odxtools.utils import ref
from odxtools.diagdatadictionaryspec import DiagDataDictionarySpec
from enums import DocType
import warnings

def generate_can_ecu(ecu_name: str, request_id: int, response_id: int):
    print(f"Generating CAN DB for {ecu_name}")
    database = Database()
    database.short_name = ecu_name

    odx_files = [
        "base/ISO_11898_2_DWCAN.odx-cs",
        "base/ISO_15765_2.odx-cs",
        "base/ISO_15765_3_CPSS.odx-cs",
        "base/UDSOnCAN_CPS.odx-c",
    ]
    for odx_filename in odx_files:
        database.add_odx_file(odx_filename)

    database.refresh()
    doc_frags = (OdxDocFragment(ecu_name, DocType.CONTAINER),)
    dlc = DiagLayerContainer(
        odx_id=OdxLinkId(f"DLC.{ecu_name}", doc_fragments=doc_frags),
        short_name=ecu_name,
    )
    add_admin_data(dlc)
    add_company_datas(dlc)
    add_additional_audiences(dlc)

    from odxtools.comparaminstance import ComparamInstance

    # Fetch CP_UniqueRespIdTable which holds rxId and txId for CAN
    # In UDSOnCAN_CPS, it's defined in the protocol or subset
    # Wait, the generation is complex. Let's see if we can just do it simpler.
    
    # Actually, we can use the existing _add_base_variant_minimal_comparams and modify it?
    pass

if __name__ == "__main__":
    generate_can_ecu("CANTMCC3000", 0x730, 0x738)

