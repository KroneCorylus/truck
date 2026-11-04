// Supplement retained maxima with their source-edge incident UV and H1 equations.
#define H1_WITNESS_LIBRARY
#include "h1_local_witness.cpp"
int main(int argc,char**argv){
 if(argc!=6)return 2;
 TopoDS_Shape native;BRep_Builder b;if(!BRepTools::Read(native,argv[1],b))return 2;
 STEPControl_Reader reader;if(reader.ReadFile(argv[2])!=IFSelect_RetDone||!reader.TransferRoots())return 2;
 auto truck=reader.OneShape();bool fromTruck=std::string(argv[3])=="truck";auto source=fromTruck?truck:native;
 TopTools_IndexedMapOfShape edges;TopExp::MapShapes(source,TopAbs_EDGE,edges);int index=std::stoi(argv[4]);
 auto edge=TopoDS::Edge(edges(index));BRepAdaptor_Curve curve(edge);auto p=curve.Value(std::stod(argv[5]));
 auto ts=root([](double t){return gp_Vec(gp_Pnt(0,0,0),frame(t).c).Dot(M)+5;},.8,1);auto f=frame(ts);
 auto dir=gp_Vec(f.c,p).Normalized();auto projected=f.c.Translated(dir*5);
 std::cout<<std::setprecision(17)<<"source "<<argv[3]<<" edge "<<index<<" curve_type "<<curve.GetType()<<" orientation "<<edge.Orientation()<<" xyz ";point(p);
 std::cout<<" incident"<<nearest(p,source,faces(source)).detail<<" sphere_radius_residual "<<p.Distance(f.c)-5<<" sphere_canal_plane_residual "<<gp_Vec(f.c,p).Dot(f.cp)<<" phi "<<std::atan2(dir.Dot(f.v),dir.Dot(Z))<<" alpha "<<f.alpha<<'\n';
 if(fromTruck){
 std::cout<<"independent_sphere_canal_boundary ";point(projected);
 std::cout<<" source_margin "<<margin(projected)<<" native_distance "<<nearest(projected,native,faces(native)).d<<" truck_distance "<<nearest(projected,truck,faces(truck)).d;
 for(double off:{-.0003,.0003}){auto q=projected.Translated(dir*off);std::cout<<" offset "<<off<<" xyz ";point(q);std::cout<<" native "<<state(native,q)<<" truck "<<state(truck,q);}std::cout<<'\n';
 }
 return 0;
}
